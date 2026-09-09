//! [`TpcdsGenerationPlan`]: row group layout for TPC-DS Parquet files.

use std::ops::RangeInclusive;
use tpcdsgen::config::{Scaling, Table};

/// Parquet files can have at most 32767 row groups
const MAX_ROW_GROUPS: i64 = 32767;

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
    ranges: Vec<RangeInclusive<i64>>,
}

impl TpcdsGenerationPlan {
    /// Compute the row group layout for `table` given the target
    /// `row_group_bytes`.
    pub(super) fn new(table: Table, scaling: &Scaling, row_group_bytes: usize) -> Self {
        let source_rows = scaling.get_row_count(table.source_table());
        let estimated_bytes =
            (source_rows as f64 * estimated_bytes_per_source_row(table)).ceil() as i64;
        let num_row_groups = (estimated_bytes / row_group_bytes.max(1) as i64 + 1)
            .min(MAX_ROW_GROUPS)
            .min(source_rows)
            .max(1);
        // ceiling division so the last row group is the one that comes up short
        let rows_per_group = ((source_rows + num_row_groups - 1) / num_row_groups).max(1);

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
    type Item = RangeInclusive<i64>;
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
/// Measured offline at scale factor 100 using the default column encodings.
/// Large tables were sampled in single row groups near 128 MiB uncompressed
/// (119-165 MiB); smaller tables were measured in full. Sizes are approximate:
/// cardinality, row-group size, and column encodings affect encoding efficiency.
///
/// The estimates are the sum of Parquet metadata's
/// `total_uncompressed_size` divided by the exact source-row range used to
/// generate the group. Sales and returns must both use their paired sales
/// table's source-row count, not their output-row count. Fractional bytes avoid
/// large rounding errors for narrow tables such as inventory.
fn estimated_bytes_per_source_row(table: Table) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_ROW_GROUP_BYTES: usize = 7 * 1024 * 1024;

    fn plan(table: Table, scale_factor: f64, row_group_bytes: usize) -> TpcdsGenerationPlan {
        TpcdsGenerationPlan::new(table, &Scaling::new(scale_factor), row_group_bytes)
    }

    /// Assert the ranges cover `1..=expected_source_rows` contiguously
    fn assert_covers(plan: &TpcdsGenerationPlan, expected_source_rows: i64) {
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
        assert_eq!(plan.row_group_count(), 19);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn narrow_tables_keep_fractional_byte_estimates() {
        let plan = plan(Table::Inventory, 100.0, 128 * 1024 * 1024);
        // Rounding 3.45 bytes/source row to an integer would produce 9 or 12 groups.
        assert_eq!(plan.row_group_count(), 11);
        assert_covers(&plan, Scaling::new(100.0).get_row_count(Table::Inventory));
    }

    #[test]
    fn store_returns_ranges_use_sales_source_rows() {
        let plan = plan(Table::StoreReturns, 1.0, DEFAULT_ROW_GROUP_BYTES);
        // store_returns is generated from the 240k store_sales source rows
        // (its own scaling row count is 0)
        assert_eq!(plan.row_group_count(), 3);
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
