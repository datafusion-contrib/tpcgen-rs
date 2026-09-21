//! [`TpcdsGenerationPlan`]: how a TPC-DS table is split into chunks.

use super::parquet::MAX_ROW_GROUPS;
use std::ops::RangeInclusive;
use tpcdsgen::config::Table;

/// What a chunk of a table is generated into.
///
/// Selects the estimated output bytes per source row, and how many chunks a
/// table may be split into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChunkFormat {
    /// One chunk is one Parquet row group
    Parquet,
    /// One chunk is one in memory buffer of DAT text
    Dat,
    /// One chunk is one in memory buffer of CSV text
    Csv,
}

/// How to generate a TPC-DS table: a list of contiguous source row ranges,
/// each of which is generated as one chunk (a Parquet row group, or a buffer
/// of DAT or CSV text). Each range can be generated independently, in
/// parallel.
///
/// The number of chunks is computed from the source row count, an estimated
/// output size per source row, and the target chunk size.
///
/// Note the ranges are over *source* rows, which is not the same as output rows
/// for all tables: for example, the sales generators emit several output rows
/// per source row, and the returns tables are generated from their paired sales
/// generator, so their ranges are over the *sales* source rows.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TpcdsGenerationPlan {
    /// Inclusive 1-based source row ranges, one per chunk
    ranges: Vec<RangeInclusive<u64>>,
}

impl TpcdsGenerationPlan {
    /// Compute the chunk layout for `table` given the target
    /// `chunk_size_bytes` of generated `format` output, restricted to
    /// `row_range` of the table's source rows.
    ///
    /// `row_range` is typically a whole table (`1..=source_rows`) or one
    /// `--parts`/`--part` chunk (see
    /// [`tpcdsgen::config::Session::get_source_row_range`]); either way the
    /// chunks it produces cover exactly `row_range`.
    pub(super) fn new_for_range(
        table: Table,
        chunk_size_bytes: i64,
        row_range: RangeInclusive<u64>,
        format: ChunkFormat,
    ) -> Self {
        let range_start = *row_range.start();
        let range_end = *row_range.end();
        let range_len = if range_end >= range_start {
            range_end - range_start + 1
        } else {
            0
        };

        let max_chunks = match format {
            ChunkFormat::Parquet => MAX_ROW_GROUPS,
            // Text chunks are buffers, so there is no limit on how many there
            // can be. Capping them would instead grow the buffers, and with
            // them peak memory use, at high scale factors.
            ChunkFormat::Dat | ChunkFormat::Csv => u64::MAX,
        };
        let estimated_bytes =
            (range_len as f64 * estimated_bytes_per_source_row(table, format)).ceil() as u64;
        let num_chunks = estimated_bytes
            .div_ceil(chunk_size_bytes.max(1) as u64)
            .min(max_chunks)
            .min(range_len)
            .max(1);
        // ceiling division so the last chunk is the one that comes up short
        let rows_per_chunk = range_len.div_ceil(num_chunks).max(1);

        let mut ranges = Vec::with_capacity(num_chunks as usize);
        let mut start = range_start;
        while start <= range_end {
            let end = (start + rows_per_chunk - 1).min(range_end);
            ranges.push(start..=end);
            start = end + 1;
        }
        // An empty range still needs one (empty) chunk so that an (empty)
        // output file is written: for Parquet a valid file with just the
        // table schema, for the text formats an empty or header only file.
        if ranges.is_empty() {
            ranges.push(row_range);
        }
        Self { ranges }
    }

    /// Return the number of chunks (for Parquet, row groups) this plan will
    /// generate
    pub(super) fn chunk_count(&self) -> usize {
        self.ranges.len()
    }
}

/// Converts the plan into an iterator of inclusive source row ranges
impl IntoIterator for TpcdsGenerationPlan {
    type Item = RangeInclusive<u64>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
    }
}

/// Estimated output bytes written per *source* row (see
/// [`TpcdsGenerationPlan`] for what a source row is).
fn estimated_bytes_per_source_row(table: Table, format: ChunkFormat) -> f64 {
    match format {
        ChunkFormat::Parquet => estimated_parquet_bytes_per_source_row(table),
        ChunkFormat::Dat => estimated_dat_bytes_per_source_row(table),
        ChunkFormat::Csv => estimated_csv_bytes_per_source_row(table),
    }
}

/// Estimated (uncompressed) Parquet bytes written per *source* row (see
/// [`TpcdsGenerationPlan`] for what a source row is).
///
/// Measured at scale factor 100 using the default column encodings.
///
/// To remeasure the estimates, first generate scale-factor-100 Parquet files
/// with approximately 128 MiB row groups:
/// ```shell
/// cargo run --release --bin tpcgen-cli -- tpcds parquet \
///   --scale-factor 100 \
///   --row-group-bytes 134217728 \
///   --output-dir /tmp/tpcds-sf100
/// cd /tmp/tpcds-sf100
/// ```
///
/// Then divide each file's total uncompressed Parquet size by its source-row
/// count. Sales generators emit multiple output rows per source row, and return
/// tables use the source rows of their paired sales table, so use distinct
/// order or ticket numbers from the sales file for both:
/// ```shell
/// for table in call_center catalog_page catalog_returns catalog_sales customer customer_address \
///   customer_demographics date_dim dbgen_version household_demographics income_band inventory \
///   item promotion reason ship_mode store store_returns store_sales time_dim warehouse web_page \
///   web_returns web_sales web_site; do
///   case "$table" in
///     catalog_sales|catalog_returns)
///       source_rows="(select count(distinct cs_order_number) from 'catalog_sales.parquet')"
///       ;;
///     store_sales|store_returns)
///       source_rows="(select count(distinct ss_ticket_number) from 'store_sales.parquet')"
///       ;;
///     web_sales|web_returns)
///       source_rows="(select count(distinct ws_order_number) from 'web_sales.parquet')"
///       ;;
///     *)
///       source_rows="(select count(*) from '$table.parquet')"
///       ;;
///   esac
///
///   datafusion-cli -q -c "
///     select
///       '$table' as table_name,
///       cast(sum(total_uncompressed_size) as double) /
///         cast($source_rows as double) as bytes_per_source_row
///     from parquet_metadata('$table.parquet')"
/// done
/// ```
///
/// The estimates are the sum of Parquet metadata's
/// `total_uncompressed_size` divided by the exact source-row range used to
/// generate the group. Sales and returns must both use their paired sales
/// table's source-row count, not their output-row count. Fractional bytes avoid
/// large rounding errors for narrow tables such as inventory.
fn estimated_parquet_bytes_per_source_row(table: Table) -> f64 {
    match table {
        Table::CallCenter => 229.80,
        Table::CatalogPage => 108.21,
        Table::CatalogReturns => 65.97,
        Table::CatalogSales => 668.72,
        Table::Customer => 76.65,
        Table::CustomerAddress => 35.63,
        Table::CustomerDemographics => 5.06,
        Table::DateDim => 52.56,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 448.00,
        Table::HouseholdDemographics => 6.44,
        Table::IncomeBand => 20.05,
        Table::Inventory => 3.45,
        Table::Item => 188.84,
        Table::Promotion => 85.67,
        Table::Reason => 45.85,
        Table::ShipMode => 71.65,
        Table::Store => 131.99,
        Table::StoreReturns => 66.27,
        Table::StoreSales => 578.47,
        Table::TimeDim => 34.03,
        Table::Warehouse => 156.93,
        Table::WebPage => 27.57,
        Table::WebReturns => 86.25,
        Table::WebSales => 799.19,
        Table::WebSite => 231.17,
        // Not a main table; never generated as Parquet output
        _ => unreachable!("Parquet generation plans are only defined for main TPC-DS tables"),
    }
}

/// Estimated DAT bytes written per *source* row (see [`TpcdsGenerationPlan`]
/// for what a source row is).
///
/// Measured at scale factor 10 as each `<table>.dat` file's size divided by
/// the table's source row count:
/// ```shell
/// cargo run --release --bin tpcgen-cli -- tpcds dat \
///   --scale-factor 10 --output-dir /tmp/tpcds-sf10-dat
/// ```
fn estimated_dat_bytes_per_source_row(table: Table) -> f64 {
    match table {
        Table::CallCenter => 314.33,
        Table::CatalogPage => 139.25,
        Table::CatalogReturns => 138.49,
        Table::CatalogSales => 1898.68,
        Table::Customer => 133.75,
        Table::CustomerAddress => 110.83,
        Table::CustomerDemographics => 41.99,
        Table::DateDim => 141.24,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 229.00,
        Table::HouseholdDemographics => 21.06,
        Table::IncomeBand => 16.40,
        Table::Inventory => 20.76,
        Table::Item => 282.90,
        Table::Promotion => 124.32,
        Table::Reason => 36.64,
        Table::ShipMode => 55.65,
        Table::Store => 265.54,
        Table::StoreReturns => 140.98,
        Table::StoreSales => 1666.22,
        Table::TimeDim => 59.12,
        Table::Warehouse => 118.60,
        Table::WebPage => 96.66,
        Table::WebReturns => 170.09,
        Table::WebSales => 2519.04,
        Table::WebSite => 289.69,
        // Not a main table; never generated as DAT output
        _ => unreachable!("DAT generation plans are only defined for main TPC-DS tables"),
    }
}

/// Estimated CSV bytes written per *source* row (see [`TpcdsGenerationPlan`]
/// for what a source row is).
///
/// Measured like [`estimated_dat_bytes_per_source_row`], from scale factor 10
/// `<table>.csv` files. CSV rows are close to DAT rows in size: they drop
/// DAT's trailing separator but quote the free text columns.
fn estimated_csv_bytes_per_source_row(table: Table) -> f64 {
    match table {
        Table::CallCenter => 334.50,
        Table::CatalogPage => 140.25,
        Table::CatalogReturns => 137.59,
        Table::CatalogSales => 1889.68,
        Table::Customer => 134.69,
        Table::CustomerAddress => 109.83,
        Table::CustomerDemographics => 40.99,
        Table::DateDim => 140.24,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 285.00,
        Table::HouseholdDemographics => 20.07,
        Table::IncomeBand => 17.80,
        Table::Inventory => 19.76,
        Table::Item => 283.89,
        Table::Promotion => 125.85,
        Table::Reason => 36.49,
        Table::ShipMode => 58.20,
        Table::Store => 270.27,
        Table::StoreReturns => 139.79,
        Table::StoreSales => 1654.22,
        Table::TimeDim => 58.12,
        Table::Warehouse => 137.10,
        Table::WebPage => 96.69,
        Table::WebReturns => 168.90,
        Table::WebSales => 2507.04,
        Table::WebSite => 301.40,
        // Not a main table; never generated as CSV output
        _ => unreachable!("CSV generation plans are only defined for main TPC-DS tables"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpcdsgen::config::Scaling;

    const DEFAULT_ROW_GROUP_BYTES: i64 = 7 * 1024 * 1024;

    fn plan(table: Table, scale_factor: f64, row_group_bytes: i64) -> TpcdsGenerationPlan {
        plan_with_format(table, scale_factor, row_group_bytes, ChunkFormat::Parquet)
    }

    fn plan_with_format(
        table: Table,
        scale_factor: f64,
        chunk_size_bytes: i64,
        format: ChunkFormat,
    ) -> TpcdsGenerationPlan {
        let source_rows = Scaling::new(scale_factor).get_row_count(table.source_table());
        TpcdsGenerationPlan::new_for_range(table, chunk_size_bytes, 1..=source_rows, format)
    }

    /// Assert the ranges cover `1..=expected_source_rows` contiguously
    fn assert_covers(plan: &TpcdsGenerationPlan, expected_source_rows: u64) {
        let mut next_row = 1;
        for range in &plan.ranges {
            assert_eq!(*range.start(), next_row);
            assert!(range.end() >= range.start());
            next_row = range.end() + 1;
        }
        assert_eq!(next_row, expected_source_rows + 1);
    }

    #[test]
    fn small_table_single_row_group() {
        let plan = plan(Table::Reason, 1.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.ranges, vec![1..=35]);
    }

    #[test]
    fn store_sales_sf1_default() {
        let plan = plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        // ~132 MiB estimated output in 7 MiB row groups over 240k source rows
        assert_eq!(plan.chunk_count(), 19);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn narrow_tables_keep_fractional_byte_estimates() {
        let plan = plan(Table::Inventory, 100.0, 128 * 1024 * 1024);
        // Rounding 3.45 bytes/source row to an integer would produce 9 or 12 groups.
        assert_eq!(plan.chunk_count(), 11);
        assert_covers(&plan, Scaling::new(100.0).get_row_count(Table::Inventory));
    }

    #[test]
    fn exact_target_multiples_do_not_add_a_row_group() {
        for (target, expected) in [(46_368, 1), (23_184, 2), (23_183, 3)] {
            let plan = plan(Table::HouseholdDemographics, 1.0, target);
            assert_eq!(plan.chunk_count(), expected);
            assert_covers(&plan, 7200);
        }
    }

    #[test]
    fn maximum_target_keeps_one_row_group() {
        let plan = plan(Table::StoreSales, 1.0, i64::MAX);
        assert_eq!(plan.chunk_count(), 1);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn store_returns_ranges_use_sales_source_rows() {
        let plan = plan(Table::StoreReturns, 1.0, DEFAULT_ROW_GROUP_BYTES);
        // store_returns is generated from the 240k store_sales source rows
        // (its own scaling row count is 0)
        assert_eq!(plan.chunk_count(), 3);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn smaller_row_groups_make_more_row_groups() {
        let default = plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        let small = plan(Table::StoreSales, 1.0, 1024 * 1024);
        assert!(small.chunk_count() > default.chunk_count());
        assert_covers(&small, 240_000);
    }

    #[test]
    fn row_group_count_is_capped() {
        let plan = plan(Table::StoreSales, 3000.0, 1024);
        // ceiling division can leave the count just under the cap
        assert!(plan.chunk_count() <= MAX_ROW_GROUPS as usize);
        assert!(plan.chunk_count() > (MAX_ROW_GROUPS - 2) as usize);
        let source_rows = Scaling::new(3000.0).get_row_count(Table::StoreSales);
        assert_covers(&plan, source_rows);
    }

    #[test]
    fn row_groups_never_exceed_source_rows() {
        // 35 source rows in 1 byte row groups still yields at most 35 groups
        let plan = plan(Table::Reason, 1.0, 1);
        assert_eq!(plan.chunk_count(), 35);
        assert_covers(&plan, 35);
    }

    #[test]
    fn non_positive_row_group_size_is_clamped() {
        let expected = plan(Table::Reason, 1.0, 1);
        for row_group_bytes in [0, -1, i64::MIN] {
            assert_eq!(plan(Table::Reason, 1.0, row_group_bytes), expected);
        }
    }

    #[test]
    fn empty_table_gets_one_empty_range() {
        let plan = plan(Table::Reason, 0.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.chunk_count(), 1);
        assert!(plan.ranges[0].is_empty());
    }

    #[test]
    fn text_chunks_are_sized_from_text_bytes() {
        // store_sales is ~381 MiB of DAT and ~379 MiB of CSV at SF 1, far
        // more than its ~132 MiB of uncompressed Parquet.
        let parquet = plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        for format in [ChunkFormat::Dat, ChunkFormat::Csv] {
            let text = plan_with_format(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES, format);
            assert!(text.chunk_count() > parquet.chunk_count(), "{format:?}");
            assert_covers(&text, 240_000);
        }
    }

    #[test]
    fn text_chunk_count_is_not_capped_at_the_parquet_row_group_limit() {
        for format in [ChunkFormat::Dat, ChunkFormat::Csv] {
            let plan = plan_with_format(Table::StoreSales, 3000.0, DEFAULT_ROW_GROUP_BYTES, format);
            assert!(plan.chunk_count() > MAX_ROW_GROUPS as usize);
            assert_covers(&plan, Scaling::new(3000.0).get_row_count(Table::StoreSales));
        }
    }

    #[test]
    fn text_returns_ranges_use_sales_source_rows() {
        for format in [ChunkFormat::Dat, ChunkFormat::Csv] {
            let plan = plan_with_format(Table::StoreReturns, 1.0, DEFAULT_ROW_GROUP_BYTES, format);
            assert_covers(&plan, 240_000);
        }
    }
}
