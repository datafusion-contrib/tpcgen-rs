//! [`TpcdsGenerationPlan`]: row group layout for TPC-DS Parquet files.

use std::ops::RangeInclusive;
use tpcdsgen::config::{Scaling, Table};

/// Parquet files can have at most 32767 row groups
const MAX_ROW_GROUPS: u64 = 32767;

/// How to generate a TPC-DS table as a Parquet file: a list of contiguous
/// source row ranges, each of which is generated as one row group.
///
/// The number of row groups is computed from the source row count, an estimated
/// Parquet bytes per source row, and the target row group size, capped at
/// Parquet's row group limit. Each range can then be generated (and encoded)
/// independently, in parallel.
///
/// Note the ranges are over *source* rows, which is not the same as output rows
/// for all tables: for example, the sales generators emit several output rows
/// per source row, and the returns tables are generated from their paired sales
/// generator, so their ranges are over the *sales* source rows.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TpcdsGenerationPlan {
    /// Inclusive 1-based source row ranges, one per row group
    ranges: Vec<RangeInclusive<u64>>,
}

impl TpcdsGenerationPlan {
    /// Compute the row group layout for `table` given the target
    /// `row_group_bytes`.
    pub(super) fn new(table: Table, scaling: &Scaling, row_group_bytes: usize) -> Self {
        let source_rows = scaling.get_row_count(table.source_table());
        let estimated_bytes = source_rows.saturating_mul(estimated_bytes_per_source_row(table));
        let num_row_groups = (estimated_bytes / row_group_bytes.max(1) as u64 + 1)
            .min(MAX_ROW_GROUPS)
            .min(source_rows)
            .max(1);
        // ceiling division so the last row group is the one that comes up short
        let rows_per_group = source_rows.div_ceil(num_row_groups).max(1);

        let mut ranges = Vec::with_capacity(num_row_groups as usize);
        let mut start = 1;
        while start <= source_rows {
            let end = (start + rows_per_group - 1).min(source_rows);
            ranges.push(start..=end);
            start = end + 1;
        }
        // An empty table still needs one (empty) range so that a valid
        // Parquet file containing the table schema is written.
        if ranges.is_empty() {
            #[allow(clippy::reversed_empty_ranges)]
            ranges.push(1..=0);
        }
        Self { ranges }
    }

    /// Return the number of row groups this plan will generate
    pub(super) fn row_group_count(&self) -> usize {
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

/// Estimated (uncompressed) Parquet bytes written per *source* row (see
/// [`TpcdsGenerationPlan`] for what a source row is).
///
/// Row group sizes are conventionally measured in uncompressed bytes, which
/// is also what the previous `ArrowWriter` based implementation limited.
///
/// Measured from files generated at scale factor 1: the total uncompressed
/// bytes, computed using datafusion-cli:
///
/// You can verify these numbers using
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
///   select
///     '$table' as table_name,
///     round(
///       cast(sum(total_uncompressed_size) as double) / cast($source_rows as double)
///     ) as bytes_per_source_row
///   from parquet_metadata('$table.parquet')"
/// done
/// ```
///
/// Which results in something like
/// ```text
/// +-------------+----------------------+
/// | table_name  | bytes_per_source_row |
/// +-------------+----------------------+
/// | call_center | 423.0                |
/// +-------------+----------------------+
/// ...
/// +-----------------+----------------------+
/// | table_name      | bytes_per_source_row |
/// +-----------------+----------------------+
/// | catalog_returns | 195.0                |
/// +-----------------+----------------------+
/// +---------------+----------------------+
/// | table_name    | bytes_per_source_row |
/// +---------------+----------------------+
/// | catalog_sales | 2391.0               |
/// +---------------+----------------------+
/// ```
///
/// Remember you have to divide by the **source** row count (which is different
/// for sales vs returns tables) to get the bytes per source row.
fn estimated_bytes_per_source_row(table: Table) -> u64 {
    match table {
        Table::CallCenter => 423,
        Table::CatalogPage => 113,
        Table::CatalogReturns => 195,
        Table::CatalogSales => 2391,
        Table::Customer => 92,
        Table::CustomerAddress => 46,
        Table::CustomerDemographics => 9,
        Table::DateDim => 57,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 358,
        Table::HouseholdDemographics => 10,
        Table::IncomeBand => 20,
        Table::Inventory => 3,
        Table::Item => 197,
        Table::Promotion => 120,
        Table::Reason => 54,
        Table::ShipMode => 76,
        Table::Store => 265,
        Table::StoreReturns => 220,
        Table::StoreSales => 2366,
        Table::TimeDim => 38,
        Table::Warehouse => 206,
        Table::WebPage => 50,
        Table::WebReturns => 261,
        Table::WebSales => 3119,
        Table::WebSite => 218,
        // Not a main table; never generated as Parquet output
        _ => unreachable!("Parquet generation plans are only defined for main TPC-DS tables"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_ROW_GROUP_BYTES: usize = 7 * 1024 * 1024;

    fn plan(table: Table, scale_factor: f64, row_group_bytes: usize) -> TpcdsGenerationPlan {
        TpcdsGenerationPlan::new(table, &Scaling::new(scale_factor), row_group_bytes)
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
        // ~568 MB estimated output in 7 MB row groups over 240k source rows
        assert_eq!(plan.row_group_count(), 78);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn store_returns_ranges_use_sales_source_rows() {
        let plan = plan(Table::StoreReturns, 1.0, DEFAULT_ROW_GROUP_BYTES);
        // store_returns is generated from the 240k store_sales source rows
        // (its own scaling row count is 0)
        assert_eq!(plan.row_group_count(), 8);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn smaller_row_groups_make_more_row_groups() {
        let default = plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        let small = plan(Table::StoreSales, 1.0, 1024 * 1024);
        assert!(small.row_group_count() > default.row_group_count());
        assert_covers(&small, 240_000);
    }

    #[test]
    fn row_group_count_is_capped() {
        let plan = plan(Table::StoreSales, 3000.0, 1024);
        // ceiling division can leave the count just under the cap
        assert!(plan.row_group_count() <= MAX_ROW_GROUPS as usize);
        assert!(plan.row_group_count() > (MAX_ROW_GROUPS - 2) as usize);
        let source_rows = Scaling::new(3000.0).get_row_count(Table::StoreSales);
        assert_covers(&plan, source_rows);
    }

    #[test]
    fn row_groups_never_exceed_source_rows() {
        // 35 source rows in 1 byte row groups still yields at most 35 groups
        let plan = plan(Table::Reason, 1.0, 1);
        assert_eq!(plan.row_group_count(), 35);
        assert_covers(&plan, 35);
    }

    #[test]
    fn empty_table_gets_one_empty_range() {
        let plan = plan(Table::Reason, 0.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.row_group_count(), 1);
        assert!(plan.ranges[0].is_empty());
    }
}
