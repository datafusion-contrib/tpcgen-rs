use crate::config::{CompatMode, Table};
use crate::distribution::calendar_distribution::{CalendarDistribution, CalendarWeights};
use crate::table::Table as MetaTable;
use crate::types::Date;

/// Row count for the `reason` table under C dsdgen compatibility.
///
/// C dsdgen generates all 75 entries from `return_reasons_c.dst` at every
/// scale factor. The Java/Trino port incorrectly used a logarithmic scaling
/// model starting at 35 for SF1; this constant fixes that when
/// `CompatMode::C` is active, see [1].
///
/// [1]: https://github.com/trinodb/tpcds/blob/8a02abbba864feedc2afd078c8153d66a95bb2d4/src/main/java/io/trino/tpcds/Table.java#L201
const REASON_ROW_COUNT_C: u64 = 75;

/// Number of tables with precomputed row counts: the main output tables,
/// which are the first `CACHED_TABLE_COUNT` variants of [`Table`]
/// (`CallCenter..=DbgenVersion`), so a table's discriminant is its index.
const CACHED_TABLE_COUNT: usize = 25;

#[derive(Debug, Clone)]
pub struct Scaling {
    scale: f64,
    compat_mode: CompatMode,
    /// Row counts for the main output tables, indexed by `Table`
    /// discriminant. `get_row_count` is called for every foreign key
    /// generated, so it must be a plain lookup rather than a repeated
    /// ScalingInfo model computation.
    row_counts: [u64; CACHED_TABLE_COUNT],
}

impl Scaling {
    pub fn new(scale: f64) -> Self {
        Self::new_with_compat(scale, CompatMode::default())
    }

    pub fn new_with_compat(scale: f64, compat_mode: CompatMode) -> Self {
        let mut scaling = Scaling {
            scale,
            compat_mode,
            row_counts: [0; CACHED_TABLE_COUNT],
        };
        for table in Table::main_tables() {
            // Inventory derives from the Item and Warehouse entries, so it
            // is filled once the rest of the cache is populated.
            if table != Table::Inventory {
                scaling.row_counts[table as usize] = scaling.compute_row_count(table);
            }
        }
        scaling.row_counts[Table::Inventory as usize] = scaling.scale_inventory();
        scaling
    }

    pub fn get_scale(&self) -> f64 {
        self.scale
    }

    pub fn get_compat_mode(&self) -> CompatMode {
        self.compat_mode
    }

    /// Get row count for a table at this scale factor.
    ///
    /// Main output tables are answered from the cache built at construction;
    /// only source tables fall through to the model computation (where they
    /// panic, as before).
    pub fn get_row_count(&self, table: Table) -> u64 {
        let index = table as usize;
        if index < CACHED_TABLE_COUNT {
            return self.row_counts[index];
        }
        self.compute_row_count(table)
    }

    /// Compute a table's row count from its ScalingInfo model (Static,
    /// Linear, or Logarithmic). Used to populate the cache.
    ///
    /// Note: Inventory is a special case - its row count is computed dynamically
    /// as item_id_count × warehouse_count × weeks (matching Java's scaleInventory()).
    fn compute_row_count(&self, table: Table) -> u64 {
        // Special case for Inventory - computed dynamically like Java's scaleInventory()
        // See: Java Scaling.java getRowCount() and scaleInventory()
        if table == Table::Inventory {
            return self.scale_inventory();
        }

        // In C compat mode the reason table is static at 75 rows regardless of scale.
        // The Java port's logarithmic model (35 at SF1) is wrong; C always emits all
        // 75 entries from return_reasons_c.dst.
        if table == Table::Reason && self.compat_mode == CompatMode::C {
            return REASON_ROW_COUNT_C;
        }

        // Convert config::Table to table::Table to access ScalingInfo
        let meta_table = Self::to_meta_table(table);

        // Get base row count from ScalingInfo
        let scaling_info = meta_table.get_scaling_info();
        let base_row_count: u64 = scaling_info
            .get_row_count_for_scale(self.scale)
            .unwrap_or(0)
            .try_into()
            .expect("row count cannot be negative");

        // Apply multiplier based on keepsHistory and scalingInfo.multiplier
        // multiplier = (keepsHistory ? 2 : 1) * 10^scalingInfo.multiplier
        let mut multiplier: u64 = if meta_table.keeps_history() { 2 } else { 1 };
        for _ in 0..scaling_info.get_multiplier() {
            multiplier *= 10;
        }

        base_row_count * multiplier
    }

    /// Compute inventory row count dynamically.
    ///
    /// Inventory row count = item_id_count × warehouse_count × weeks
    /// This matches Java's Scaling.scaleInventory() method exactly.
    ///
    /// From Java:
    /// ```java
    /// private long scaleInventory() {
    ///     int nDays = JULIAN_DATE_MAXIMUM - JULIAN_DATE_MINIMUM;
    ///     nDays += 7;  // ndays + 1 + 6
    ///     nDays /= 7;  // each item's inventory is updated weekly
    ///     return getIdCount(ITEM) * getRowCount(WAREHOUSE) * nDays;
    /// }
    /// ```
    fn scale_inventory(&self) -> u64 {
        let n_days = Date::JULIAN_DATE_MAXIMUM - Date::JULIAN_DATE_MINIMUM;
        let n_weeks = (n_days + 7) / 7; // Round up to weeks
        self.get_id_count(Table::Item) * self.get_row_count(Table::Warehouse) * n_weeks as u64
    }

    /// Convert config::Table to table::Table for accessing metadata
    fn to_meta_table(table: Table) -> MetaTable {
        match table {
            Table::CallCenter => MetaTable::CallCenter,
            Table::CatalogPage => MetaTable::CatalogPage,
            Table::CatalogReturns => MetaTable::CatalogReturns,
            Table::CatalogSales => MetaTable::CatalogSales,
            Table::Customer => MetaTable::Customer,
            Table::CustomerAddress => MetaTable::CustomerAddress,
            Table::CustomerDemographics => MetaTable::CustomerDemographics,
            Table::DateDim => MetaTable::DateDim,
            Table::HouseholdDemographics => MetaTable::HouseholdDemographics,
            Table::IncomeBand => MetaTable::IncomeBand,
            Table::Inventory => MetaTable::Inventory,
            Table::Item => MetaTable::Item,
            Table::Promotion => MetaTable::Promotion,
            Table::Reason => MetaTable::Reason,
            Table::ShipMode => MetaTable::ShipMode,
            Table::Store => MetaTable::Store,
            Table::StoreReturns => MetaTable::StoreReturns,
            Table::StoreSales => MetaTable::StoreSales,
            Table::TimeDim => MetaTable::TimeDim,
            Table::Warehouse => MetaTable::Warehouse,
            Table::WebPage => MetaTable::WebPage,
            Table::WebReturns => MetaTable::WebReturns,
            Table::WebSales => MetaTable::WebSales,
            Table::WebSite => MetaTable::WebSite,
            Table::DbgenVersion => MetaTable::DbgenVersion,
            // Source tables - use a default or panic
            _ => panic!(
                "Source tables not supported for row count scaling: {:?}",
                table
            ),
        }
    }

    /// Get unique ID count for tables that keep history
    pub fn get_id_count(&self, table: Table) -> u64 {
        let row_count = self.get_row_count(table);
        if table.keeps_history() {
            let unique_count = (row_count / 6) * 3;
            match row_count % 6 {
                1 => unique_count + 1,
                2 | 3 => unique_count + 2,
                4 | 5 => unique_count + 3,
                _ => unique_count,
            }
        } else {
            row_count
        }
    }

    /// Get row count for a specific date for date-based tables.
    ///
    /// For sales tables (STORE_SALES, CATALOG_SALES, WEB_SALES), this calculates
    /// how many rows to generate for a given julian date using the calendar
    /// distribution weights.
    ///
    /// Based on Scaling.getRowCountForDate in Java.
    pub fn get_row_count_for_date(&self, table: Table, julian_date: i64) -> u64 {
        let row_count = match table {
            Table::StoreSales | Table::CatalogSales | Table::WebSales => self.get_row_count(table),
            Table::Inventory => {
                self.get_row_count(Table::Warehouse) * self.get_id_count(Table::Item)
            }
            _ => panic!("Invalid table for date scaling: {:?}", table),
        };

        // Convert julian date to a Date
        let date = Date::from_julian_days(julian_date as i32);

        // Get the appropriate weights based on year (leap year or not)
        let weights = if Date::is_leap_year(date.year()) {
            CalendarWeights::SalesLeapYear
        } else {
            CalendarWeights::Sales
        };

        // Calculate row count for this date using calendar distribution
        // The formula: rowCount = (rowCount * dayWeight + calendarTotal/2) / calendarTotal
        // This distributes the total row count across dates based on weights
        let calendar_total = CalendarDistribution::get_max_weight(weights) as u64 * 5; // 5 years of data
        let day_index = CalendarDistribution::get_index_for_date(&date);
        let day_weight = CalendarDistribution::get_weight_for_day_number(day_index, weights) as u64;

        let mut result = row_count * day_weight;
        result += calendar_total / 2; // rounding
        result /= calendar_total;

        result
    }

    /// Basic row counts per table at scale factor 1.
    #[allow(dead_code)]
    fn get_base_row_count(&self, table: Table) -> i64 {
        match table {
            Table::CallCenter => 6,
            Table::CatalogPage => 11718,
            Table::CatalogReturns => 160000, // Same as CatalogSales orders (returns are ~10% of sales)
            Table::CatalogSales => 160000,   // Number of ORDERS, not line items (16 * 10^4)
            Table::Customer => 100000,
            Table::CustomerAddress => 50000,
            Table::CustomerDemographics => 1920800,
            Table::DateDim => 73049,
            Table::HouseholdDemographics => 7200,
            Table::IncomeBand => 20,
            Table::Inventory => 11745000,
            Table::Item => 18000,
            Table::Promotion => 300,
            Table::Reason => 35,
            Table::ShipMode => 20,
            Table::Store => 12,
            Table::StoreReturns => 240000, // Same as StoreSales orders (returns are ~10% of sales)
            Table::StoreSales => 240000,   // Number of ORDERS, not line items (24 * 10^4)
            Table::TimeDim => 86400,
            Table::Warehouse => 5,
            Table::WebPage => 60,
            Table::WebReturns => 60000, // Same as WebSales orders (returns are ~10% of sales)
            Table::WebSales => 60000,   // Number of ORDERS, not line items (60 * 10^3)
            Table::WebSite => 30,
            Table::DbgenVersion => 1,
            Table::SBrand => 1000,
            Table::SCustomerAddress => 50000,
            Table::SCallCenter => 6,
            Table::SCatalog => 100,
            Table::SCatalogOrder => 100000,
            Table::SCatalogOrderLineitem => 500000,
            Table::SCatalogPage => 11718,
            Table::SCatalogPromotionalItem => 10000,
            Table::SCatalogReturns => 144,
            Table::SCategory => 100,
            Table::SClass => 100,
            Table::SCompany => 100,
            Table::SCustomer => 100000,
            Table::SInventory => 1000000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaling_creation() {
        let scaling = Scaling::new(1.0);
        assert_eq!(scaling.get_scale(), 1.0);
    }

    #[test]
    fn test_row_count_calculation() {
        let scaling = Scaling::new(2.0);

        // Row count should scale with scale factor (not linear)
        let customer_rows = scaling.get_row_count(Table::Customer);
        assert_eq!(customer_rows, 144000);

        let store_rows = scaling.get_row_count(Table::Store);
        assert_eq!(store_rows, 22);
    }

    #[test]
    fn test_id_count_for_history_tables() {
        let scaling = Scaling::new(1.0);

        // Non-history table: ID count equals row count
        let customer_ids = scaling.get_id_count(Table::Customer);
        let customer_rows = scaling.get_row_count(Table::Customer);
        assert_eq!(customer_ids, customer_rows);

        // History table: ID count is less than row count
        let item_ids = scaling.get_id_count(Table::Item);
        let item_rows = scaling.get_row_count(Table::Item);
        assert!(item_ids <= item_rows);
    }

    #[test]
    fn test_fractional_scaling() {
        let scaling = Scaling::new(0.1);
        let customer_rows = scaling.get_row_count(Table::Customer);
        assert_eq!(customer_rows, 10000); // 100000 * 0.1
    }

    #[test]
    fn test_cached_row_counts_match_direct_computation() {
        for scale in [0.1, 1.0, 10.0] {
            let scaling = Scaling::new(scale);
            for table in Table::main_tables() {
                let expected = if table == Table::Inventory {
                    scaling.scale_inventory()
                } else {
                    scaling.compute_row_count(table)
                };
                assert_eq!(
                    scaling.get_row_count(table),
                    expected,
                    "cached row count diverges from model for {table:?} at scale {scale}"
                );
            }
        }
    }

    #[test]
    fn test_inventory_scaling() {
        // Inventory is computed as: item_id_count × warehouse_count × weeks
        // weeks = (JULIAN_DATE_MAXIMUM - JULIAN_DATE_MINIMUM + 7) / 7

        // Scale 1: 9000 items × 5 warehouses × 261 weeks = 11,745,000
        let scaling_1 = Scaling::new(1.0);
        let inventory_rows_1 = scaling_1.get_row_count(Table::Inventory);
        assert_eq!(inventory_rows_1, 11_745_000);

        // Scale 10: Should scale with item_id_count and warehouse_count
        let scaling_10 = Scaling::new(10.0);
        let inventory_rows_10 = scaling_10.get_row_count(Table::Inventory);

        // At scale 10:
        // - Items: 102,000 rows → id_count = 51,000 (keeps history)
        // - Warehouses: 10 rows
        // - Weeks: 261
        // Expected: 51,000 × 10 × 261 = 133,110,000
        assert_eq!(inventory_rows_10, 133_110_000);
    }
}
