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
            source_rows.saturating_mul(estimated_bytes_per_source_row(table, row_group_bytes));
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
/// The baseline estimates were measured from scale-factor-1 files and are
/// accurate for the CLI's default and smaller row groups. For large row-group
/// targets, selected tables blend toward measurements from approximately 128
/// MiB groups at scale factor 1000. Dictionary and page overhead per source
/// row is substantially lower at that size, so applying the small-group
/// estimate would create too many undersized groups.
///
/// The estimates are the sum of Parquet metadata's
/// `total_uncompressed_size` divided by the exact source-row range used to
/// generate the group. Sales and returns must both use their paired sales
/// table's source-row count, not their output-row count.
fn estimated_bytes_per_source_row(table: Table, row_group_bytes: usize) -> i64 {
    const SMALL_ROW_GROUP_BYTES: usize = 8 * 1024 * 1024;
    const LARGE_ROW_GROUP_BYTES: usize = 128 * 1024 * 1024;

    let small_group_estimate = match table {
        Table::CallCenter => 406,
        Table::CatalogPage => 108,
        Table::CatalogReturns => 79,
        Table::CatalogSales => 786,
        Table::Customer => 86,
        Table::CustomerAddress => 42,
        Table::CustomerDemographics => 5,
        Table::DateDim => 53,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 407,
        Table::HouseholdDemographics => 6,
        Table::IncomeBand => 20,
        Table::Inventory => 3,
        Table::Item => 165,
        Table::Promotion => 90,
        Table::Reason => 50,
        Table::ShipMode => 72,
        Table::Store => 248,
        Table::StoreReturns => 78,
        Table::StoreSales => 631,
        Table::TimeDim => 34,
        Table::Warehouse => 202,
        Table::WebPage => 42,
        Table::WebReturns => 104,
        Table::WebSales => 963,
        Table::WebSite => 198,
        // Not a main table; never generated as Parquet output
        _ => unreachable!("Parquet generation plans are only defined for main TPC-DS tables"),
    };
    let large_group_estimate = match table {
        Table::CatalogReturns => 67,
        Table::CatalogSales => 669,
        Table::Customer => 78,
        Table::CustomerAddress => 35,
        Table::Inventory => 4,
        Table::Item => 191,
        Table::StoreReturns => 68,
        Table::StoreSales => 594,
        Table::WebReturns => 88,
        Table::WebSales => 806,
        _ => return small_group_estimate,
    };

    if row_group_bytes <= SMALL_ROW_GROUP_BYTES {
        return small_group_estimate;
    }
    if row_group_bytes >= LARGE_ROW_GROUP_BYTES {
        return large_group_estimate;
    }

    let range = (LARGE_ROW_GROUP_BYTES - SMALL_ROW_GROUP_BYTES) as i64;
    let progress = (row_group_bytes - SMALL_ROW_GROUP_BYTES) as i64;
    small_group_estimate + (large_group_estimate - small_group_estimate) * progress / range
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
        // ~144 MiB estimated output in 7 MiB row groups over 240k source rows
        assert_eq!(plan.row_group_count(), 21);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn large_row_groups_use_target_sized_estimates() {
        assert_eq!(
            estimated_bytes_per_source_row(Table::CatalogReturns, 128 * 1024 * 1024),
            67
        );
        assert_eq!(
            estimated_bytes_per_source_row(Table::CatalogSales, 128 * 1024 * 1024),
            669
        );
        assert_eq!(
            estimated_bytes_per_source_row(Table::Customer, 128 * 1024 * 1024),
            78
        );
        assert_eq!(
            estimated_bytes_per_source_row(Table::StoreSales, 128 * 1024 * 1024),
            594
        );
        assert_eq!(
            estimated_bytes_per_source_row(Table::WebSales, 128 * 1024 * 1024),
            806
        );
    }

    #[test]
    fn sf1000_large_row_group_counts_use_large_group_calibration() {
        let row_group_bytes = 128 * 1024 * 1024;
        for (table, expected) in [
            (Table::CatalogReturns, 80),
            (Table::CatalogSales, 798),
            (Table::Customer, 7),
            (Table::CustomerAddress, 2),
            (Table::CustomerDemographics, 1),
            (Table::Inventory, 24),
            (Table::Item, 1),
            (Table::StoreReturns, 122),
            (Table::StoreSales, 1063),
            (Table::WebReturns, 40),
            (Table::WebSales, 361),
        ] {
            assert_eq!(
                plan(table, 1000.0, row_group_bytes).row_group_count(),
                expected,
                "unexpected row-group count for {}",
                table.get_name()
            );
        }
    }

    #[test]
    fn small_row_groups_keep_small_group_estimates() {
        assert_eq!(
            estimated_bytes_per_source_row(Table::Customer, 1024 * 1024),
            86
        );
        assert_eq!(
            estimated_bytes_per_source_row(Table::StoreSales, DEFAULT_ROW_GROUP_BYTES),
            631
        );
    }

    #[test]
    fn medium_row_groups_blend_between_calibrations() {
        let medium = estimated_bytes_per_source_row(Table::Customer, 64 * 1024 * 1024);
        assert!(medium < 86);
        assert!(medium > 78);
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
