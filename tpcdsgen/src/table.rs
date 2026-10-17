use crate::scaling_info::{ScalingInfo, ScalingModel};
use crate::table_flags::TableFlags;
use crate::TpcdsError;
use std::str::FromStr;
use std::sync::OnceLock;

/// Table enum representing all TPC-DS tables with complete metadata (Table)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Table {
    CallCenter,
    CatalogPage,
    CatalogReturns,
    CatalogSales,
    Warehouse,
    ShipMode,
    Reason,
    IncomeBand,
    HouseholdDemographics,
    CustomerDemographics,
    CustomerAddress,
    Customer,
    DateDim,
    TimeDim,
    Item,
    Promotion,
    Store,
    StoreReturns,
    StoreSales,
    WebPage,
    WebReturns,
    WebSales,
    WebSite,
    DbgenVersion,
    Inventory,
}

impl Table {
    /// All TPC-DS tables
    pub fn values() -> &'static [Table] {
        &[
            Table::CallCenter,
            Table::CatalogPage,
            Table::CatalogReturns,
            Table::CatalogSales,
            Table::Customer,
            Table::CustomerAddress,
            Table::CustomerDemographics,
            Table::DateDim,
            Table::HouseholdDemographics,
            Table::IncomeBand,
            Table::Inventory,
            Table::Item,
            Table::Promotion,
            Table::Reason,
            Table::ShipMode,
            Table::Store,
            Table::StoreReturns,
            Table::StoreSales,
            Table::TimeDim,
            Table::Warehouse,
            Table::WebPage,
            Table::WebReturns,
            Table::WebSales,
            Table::WebSite,
            Table::DbgenVersion,
        ]
    }

    /// Get the lowercase table name (getName())
    pub fn get_name(&self) -> &'static str {
        match self {
            Table::CallCenter => "call_center",
            Table::CatalogPage => "catalog_page",
            Table::CatalogReturns => "catalog_returns",
            Table::CatalogSales => "catalog_sales",
            Table::Warehouse => "warehouse",
            Table::ShipMode => "ship_mode",
            Table::Reason => "reason",
            Table::IncomeBand => "income_band",
            Table::HouseholdDemographics => "household_demographics",
            Table::CustomerDemographics => "customer_demographics",
            Table::CustomerAddress => "customer_address",
            Table::Customer => "customer",
            Table::DateDim => "date_dim",
            Table::TimeDim => "time_dim",
            Table::Item => "item",
            Table::Promotion => "promotion",
            Table::Store => "store",
            Table::StoreReturns => "store_returns",
            Table::StoreSales => "store_sales",
            Table::WebPage => "web_page",
            Table::WebReturns => "web_returns",
            Table::WebSales => "web_sales",
            Table::WebSite => "web_site",
            Table::DbgenVersion => "dbgen_version",
            Table::Inventory => "inventory",
        }
    }

    /// Get the Java Table enum ordinal
    /// This MUST match the exact order in Java's Table.java enum declaration
    /// Used for SCD date calculations (table_number * 6 offset)
    pub fn get_ordinal(&self) -> i64 {
        match self {
            // Java enum order (from Table.java):
            Table::CallCenter => 0,
            Table::CatalogPage => 1,
            Table::CatalogReturns => 2,
            Table::CatalogSales => 3,
            Table::Customer => 4,
            Table::CustomerAddress => 5,
            Table::CustomerDemographics => 6,
            Table::DateDim => 7,
            Table::HouseholdDemographics => 8,
            Table::IncomeBand => 9,
            Table::Inventory => 10,
            Table::Item => 11,
            Table::Promotion => 12,
            Table::Reason => 13,
            Table::ShipMode => 14,
            Table::Store => 15,
            Table::StoreReturns => 16,
            Table::StoreSales => 17,
            Table::TimeDim => 18,
            Table::Warehouse => 19,
            Table::WebPage => 20,
            Table::WebReturns => 21,
            Table::WebSales => 22,
            Table::WebSite => 23,
            Table::DbgenVersion => 24,
        }
    }

    /// Get table flags using const static array for efficiency.
    /// Format: TableFlags::new(keeps_history, is_small, is_date_based)
    pub fn get_table_flags(&self) -> &'static TableFlags {
        // Const static array indexed by enum variant order (not Java ordinal)
        // Order matches enum definition: CallCenter, CatalogPage, ..., Inventory
        static TABLE_FLAGS: [TableFlags; 25] = [
            TableFlags::new(true, true, false), // CallCenter: keeps_history, is_small
            TableFlags::new(false, false, false), // CatalogPage: default
            TableFlags::new(false, false, false), // CatalogReturns: default
            TableFlags::new(false, false, true), // CatalogSales: is_date_based
            TableFlags::new(false, true, false), // Warehouse: is_small
            TableFlags::new(false, true, false), // ShipMode: is_small
            TableFlags::new(false, true, false), // Reason: is_small
            TableFlags::new(false, true, false), // IncomeBand: is_small
            TableFlags::new(false, false, false), // HouseholdDemographics: default
            TableFlags::new(false, false, false), // CustomerDemographics: default
            TableFlags::new(false, false, false), // CustomerAddress: default
            TableFlags::new(false, false, false), // Customer: default
            TableFlags::new(false, false, false), // DateDim: default
            TableFlags::new(false, false, false), // TimeDim: default
            TableFlags::new(true, false, false), // Item: keeps_history
            TableFlags::new(false, false, false), // Promotion: default
            TableFlags::new(true, true, false), // Store: keeps_history, is_small
            TableFlags::new(false, false, false), // StoreReturns: default
            TableFlags::new(false, false, true), // StoreSales: is_date_based
            TableFlags::new(true, false, false), // WebPage: keeps_history
            TableFlags::new(false, false, false), // WebReturns: default
            TableFlags::new(false, false, true), // WebSales: is_date_based
            TableFlags::new(true, true, false), // WebSite: keeps_history, is_small
            TableFlags::new(false, false, false), // DbgenVersion: default
            TableFlags::new(false, false, true), // Inventory: is_date_based
        ];

        &TABLE_FLAGS[*self as usize]
    }

    /// Get null basis points for this table
    pub fn get_null_basis_points(&self) -> i32 {
        // Const array indexed by enum variant order
        const NULL_BASIS_POINTS: [i32; 25] = [
            100,  // CallCenter
            200,  // CatalogPage
            400,  // CatalogReturns
            100,  // CatalogSales
            100,  // Warehouse
            100,  // ShipMode
            100,  // Reason
            0,    // IncomeBand
            0,    // HouseholdDemographics
            0,    // CustomerDemographics
            600,  // CustomerAddress
            700,  // Customer
            0,    // DateDim
            0,    // TimeDim
            50,   // Item
            200,  // Promotion
            100,  // Store
            700,  // StoreReturns
            900,  // StoreSales
            250,  // WebPage
            900,  // WebReturns
            5,    // WebSales
            100,  // WebSite
            0,    // DbgenVersion
            1000, // Inventory
        ];
        NULL_BASIS_POINTS[*self as usize]
    }

    /// Get not-null bitmap for this table
    pub fn get_not_null_bit_map(&self) -> i64 {
        // Const array indexed by enum variant order
        const NOT_NULL_BIT_MAP: [i64; 25] = [
            0xB,     // CallCenter
            0x3,     // CatalogPage
            0x10007, // CatalogReturns
            0x28000, // CatalogSales
            0x3,     // Warehouse
            0x3,     // ShipMode
            0x3,     // Reason
            0x1,     // IncomeBand
            0x1,     // HouseholdDemographics
            0x1,     // CustomerDemographics
            0x3,     // CustomerAddress
            0x13,    // Customer
            0x3,     // DateDim
            0x3,     // TimeDim
            0xB,     // Item
            0x3,     // Promotion
            0xB,     // Store
            0x204,   // StoreReturns
            0x204,   // StoreSales
            0xB,     // WebPage
            0x2004,  // WebReturns
            0x20008, // WebSales
            0xB,     // WebSite
            0x0,     // DbgenVersion
            0x07,    // Inventory
        ];
        NOT_NULL_BIT_MAP[*self as usize]
    }

    /// Get scaling info for this table
    pub fn get_scaling_info(&self) -> &'static ScalingInfo {
        match self {
            Table::CallCenter => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 3, 12, 15, 18, 21, 24, 27, 30, 30];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("CallCenter ScalingInfo creation should not fail")
                })
            }
            Table::CatalogPage => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 11718, 12000, 20400, 26000, 30000, 36000, 40000, 46000, 50000,
                    ];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("CatalogPage ScalingInfo creation should not fail")
                })
            }
            Table::CatalogReturns => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 16, 160, 1600, 4800, 16000, 48000, 160000, 480000, 1600000,
                    ];
                    ScalingInfo::new(4, ScalingModel::Linear, &row_counts, 0)
                        .expect("CatalogReturns ScalingInfo creation should not fail")
                })
            }
            Table::CatalogSales => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 16, 160, 1600, 4800, 16000, 48000, 160000, 480000, 1600000,
                    ];
                    ScalingInfo::new(4, ScalingModel::Linear, &row_counts, 0)
                        .expect("CatalogSales ScalingInfo creation should not fail")
                })
            }
            Table::Warehouse => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    // Java: new ScalingInfo(0, LOGARITHMIC, new int[] {0, 5, 10, 15, 17, 20, 22, 25, 27, 30}, 0)
                    let row_counts = [0, 5, 10, 15, 17, 20, 22, 25, 27, 30];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("Warehouse ScalingInfo creation should not fail")
                })
            }
            Table::ShipMode => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    // Java: new ScalingInfo(0, STATIC, new int[] {0, 20, 20, 20, 20, 20, 20, 20, 20, 20}, 0)
                    let row_counts = [0, 20, 20, 20, 20, 20, 20, 20, 20, 20];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("ShipMode ScalingInfo creation should not fail")
                })
            }
            Table::Reason => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    // Java: new ScalingInfo(0, LOGARITHMIC, new int[] {0, 35, 45, 55, 60, 65, 67, 70, 72, 75}, 0)
                    let row_counts = [0, 35, 45, 55, 60, 65, 67, 70, 72, 75];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("Reason ScalingInfo creation should not fail")
                })
            }
            Table::IncomeBand => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    // Java: new ScalingInfo(0, STATIC, new int[] {0, 20, 20, 20, 20, 20, 20, 20, 20, 20}, 0)
                    let row_counts = [0, 20, 20, 20, 20, 20, 20, 20, 20, 20];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("IncomeBand ScalingInfo creation should not fail")
                })
            }
            Table::HouseholdDemographics => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 7200, 7200, 7200, 7200, 7200, 7200, 7200, 7200, 7200];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("HouseholdDemographics ScalingInfo creation should not fail")
                })
            }
            Table::CustomerDemographics => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 19208, 19208, 19208, 19208, 19208, 19208, 19208, 19208, 19208,
                    ];
                    ScalingInfo::new(2, ScalingModel::Static, &row_counts, 0)
                        .expect("CustomerDemographics ScalingInfo creation should not fail")
                })
            }
            Table::CustomerAddress => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 50, 250, 1000, 2500, 6000, 15000, 32500, 40000, 50000];
                    ScalingInfo::new(3, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("CustomerAddress ScalingInfo creation should not fail")
                })
            }
            Table::Customer => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 100, 500, 2000, 5000, 12000, 30000, 65000, 80000, 100000];
                    ScalingInfo::new(3, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("Customer ScalingInfo creation should not fail")
                })
            }
            Table::DateDim => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 73049, 73049, 73049, 73049, 73049, 73049, 73049, 73049, 73049,
                    ];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("DateDim ScalingInfo creation should not fail")
                })
            }
            Table::TimeDim => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 86400, 86400, 86400, 86400, 86400, 86400, 86400, 86400, 86400,
                    ];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("TimeDim ScalingInfo creation should not fail")
                })
            }
            Table::Item => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 9, 51, 102, 132, 150, 180, 201, 231, 251];
                    ScalingInfo::new(3, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("Item ScalingInfo creation should not fail")
                })
            }
            Table::Promotion => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 300, 500, 1000, 1300, 1500, 1800, 2000, 2300, 2500];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("Promotion ScalingInfo creation should not fail")
                })
            }
            Table::Store => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 6, 51, 201, 402, 501, 675, 750, 852, 951];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("Store ScalingInfo creation should not fail")
                })
            }
            Table::StoreReturns => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 24, 240, 2400, 7200, 24000, 72000, 240000, 720000, 2400000,
                    ];
                    ScalingInfo::new(4, ScalingModel::Linear, &row_counts, 0)
                        .expect("StoreReturns ScalingInfo creation should not fail")
                })
            }
            Table::StoreSales => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 24, 240, 2400, 7200, 24000, 72000, 240000, 720000, 2400000,
                    ];
                    ScalingInfo::new(4, ScalingModel::Linear, &row_counts, 0)
                        .expect("StoreSales ScalingInfo creation should not fail")
                })
            }
            Table::WebPage => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 30, 100, 1020, 1302, 1500, 1800, 2001, 2301, 2502];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("WebPage ScalingInfo creation should not fail")
                })
            }
            Table::WebReturns => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 60, 600, 6000, 18000, 60000, 180000, 600000, 1800000, 6000000,
                    ];
                    ScalingInfo::new(3, ScalingModel::Linear, &row_counts, 0)
                        .expect("WebReturns ScalingInfo creation should not fail")
                })
            }
            Table::WebSales => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [
                        0, 60, 600, 6000, 18000, 60000, 180000, 600000, 1800000, 6000000,
                    ];
                    ScalingInfo::new(3, ScalingModel::Linear, &row_counts, 0)
                        .expect("WebSales ScalingInfo creation should not fail")
                })
            }
            Table::WebSite => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 15, 21, 12, 21, 27, 33, 39, 42, 48];
                    ScalingInfo::new(0, ScalingModel::Logarithmic, &row_counts, 0)
                        .expect("WebSite ScalingInfo creation should not fail")
                })
            }
            Table::DbgenVersion => {
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("DbgenVersion ScalingInfo creation should not fail")
                })
            }
            Table::Inventory => {
                // Inventory row count = item_id_count × warehouse_count × weeks
                // This is dynamically computed in the generator, these are placeholder values
                static SCALING: OnceLock<ScalingInfo> = OnceLock::new();
                SCALING.get_or_init(|| {
                    let row_counts = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
                    ScalingInfo::new(0, ScalingModel::Static, &row_counts, 0)
                        .expect("Inventory ScalingInfo creation should not fail")
                })
            }
        }
    }

    /// Check if this table keeps history
    pub fn keeps_history(&self) -> bool {
        self.get_table_flags().keeps_history()
    }

    /// Check if this is a small table
    pub fn is_small(&self) -> bool {
        self.get_table_flags().is_small()
    }

    /// Check if this table is date-based
    pub fn is_date_based(&self) -> bool {
        self.get_table_flags().is_date_based()
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

impl FromStr for Table {
    type Err = TpcdsError;

    /// Parse a table name, ignoring ASCII case.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Table::values()
            .iter()
            .copied()
            .find(|table| table.get_name().eq_ignore_ascii_case(s))
            .ok_or_else(|| TpcdsError::new(&format!("Invalid table name: {}", s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{CallCenterColumn, Column, ColumnTypeBase};

    #[test]
    fn test_table_name() {
        assert_eq!(Table::CallCenter.get_name(), "call_center");
        assert_eq!(format!("{}", Table::CallCenter), "call_center");
    }

    #[test]
    fn test_table_from_str() {
        assert_eq!(
            "CATALOG_SALES".parse::<Table>().unwrap(),
            Table::CatalogSales
        );
        assert_eq!(
            "catalog_sales".parse::<Table>().unwrap(),
            Table::CatalogSales
        );
        assert_eq!("STORE_SALES".parse::<Table>().unwrap(), Table::StoreSales);

        assert!("INVALID_TABLE".parse::<Table>().is_err());

        for &table in Table::values() {
            assert_eq!(table.get_name().parse::<Table>().unwrap(), table);
        }
    }

    #[test]
    fn test_values_are_in_table_number_order() {
        let tables = Table::values();
        assert_eq!(tables.len(), 25);
        for (i, table) in tables.iter().enumerate() {
            assert_eq!(table.get_ordinal(), i as i64, "{table}");
        }
        // These numbers offset SCD dates, so they must match the reference
        // generator.
        assert_eq!(Table::CallCenter.get_ordinal(), 0);
        assert_eq!(Table::Item.get_ordinal(), 11);
        assert_eq!(Table::Store.get_ordinal(), 15);
        assert_eq!(Table::WebPage.get_ordinal(), 20);
        assert_eq!(Table::WebSite.get_ordinal(), 23);
        assert_eq!(Table::DbgenVersion.get_ordinal(), 24);
    }

    #[test]
    fn test_table_flags() {
        let flags = Table::CallCenter.get_table_flags();
        assert!(flags.keeps_history());
        assert!(flags.is_small());
        assert!(!flags.is_date_based());
    }

    #[test]
    fn test_table_metadata() {
        assert_eq!(Table::CallCenter.get_null_basis_points(), 100);
        assert_eq!(Table::CallCenter.get_not_null_bit_map(), 0xB);
    }

    #[test]
    fn test_scaling_info() {
        let scaling = Table::CallCenter.get_scaling_info();
        assert_eq!(scaling.get_multiplier(), 0);
        assert_eq!(scaling.get_scaling_model(), ScalingModel::Logarithmic);

        // Test specific scale values from Java
        assert_eq!(scaling.get_row_count_for_scale(1.0).unwrap(), 3);
        assert_eq!(scaling.get_row_count_for_scale(100000.0).unwrap(), 30);
    }

    #[test]
    fn test_get_columns() {
        assert_eq!(CallCenterColumn::values().len(), 31);

        // Test first column
        let first_col = &CallCenterColumn::values()[0];
        assert_eq!(first_col.get_name(), "cc_call_center_sk");
        assert_eq!(first_col.get_position(), 0);

        let column_table: Table = first_col.get_table();
        assert_eq!(column_table, Table::CallCenter);
    }

    #[test]
    fn test_table_flags_methods() {
        let table = Table::CallCenter;
        assert!(table.keeps_history());
        assert!(table.is_small());
        assert!(!table.is_date_based());
    }

    #[test]
    fn test_table_conversions() {
        let table = Table::CallCenter;
        let column_table: crate::column::Table = table;
        let back_to_table: Table = column_table;
        assert_eq!(table, back_to_table);
    }

    #[test]
    fn test_column_types_integration() {
        let sk_column = CallCenterColumn::CcCallCenterSk;
        assert_eq!(sk_column.get_type().get_base(), ColumnTypeBase::Identifier);

        let name_column = CallCenterColumn::CcName;
        assert_eq!(name_column.get_type().get_base(), ColumnTypeBase::Varchar);
        assert_eq!(name_column.get_type().get_precision(), Some(50));

        let date_column = CallCenterColumn::CcRecStartDate;
        assert_eq!(date_column.get_type().get_base(), ColumnTypeBase::Date);
    }

    #[test]
    fn test_generator_vs_regular_column_count() {
        assert_eq!(CallCenterColumn::values().len(), 31); // User-visible columns
    }

    #[test]
    fn test_singleton_behavior() {
        // Test that repeated calls return the same references
        let flags1 = Table::CallCenter.get_table_flags();
        let flags2 = Table::CallCenter.get_table_flags();
        assert!(std::ptr::eq(flags1, flags2));

        let scaling1 = Table::CallCenter.get_scaling_info();
        let scaling2 = Table::CallCenter.get_scaling_info();
        assert!(std::ptr::eq(scaling1, scaling2));
    }
}
