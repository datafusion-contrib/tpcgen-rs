use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Warehouse generator columns (WarehouseGeneratorColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarehouseGeneratorColumn {
    WWarehouseSk,
    WWarehouseId,
    WWarehouseName,
    WWarehouseSqFt,
    WAddressStreetNum,
    WAddressStreetName1,
    WAddressStreetType,
    WAddressSuiteNum,
    WAddressCity,
    WAddressCounty,
    WAddressState,
    WAddressZip,
    WAddressCountry,
    WAddressGmtOffset,
    WNulls,
    WWarehouseAddress,
}

impl WarehouseGeneratorColumn {
    /// Get all generator columns in order
    pub fn values() -> &'static [WarehouseGeneratorColumn] {
        use WarehouseGeneratorColumn::*;
        static VALUES: &[WarehouseGeneratorColumn] = &[
            WWarehouseSk,
            WWarehouseId,
            WWarehouseName,
            WWarehouseSqFt,
            WAddressStreetNum,
            WAddressStreetName1,
            WAddressStreetType,
            WAddressSuiteNum,
            WAddressCity,
            WAddressCounty,
            WAddressState,
            WAddressZip,
            WAddressCountry,
            WAddressGmtOffset,
            WNulls,
            WWarehouseAddress,
        ];
        VALUES
    }

    /// Get the global column number and seeds per row for this generator column
    fn get_column_info(&self) -> (i32, u32) {
        use WarehouseGeneratorColumn::*;
        match self {
            WWarehouseSk => (351, 1),
            WWarehouseId => (352, 1),
            WWarehouseName => (353, 80),
            WWarehouseSqFt => (354, 1),
            WAddressStreetNum => (355, 1),
            WAddressStreetName1 => (356, 1),
            WAddressStreetType => (357, 1),
            WAddressSuiteNum => (358, 1),
            WAddressCity => (359, 1),
            WAddressCounty => (360, 1),
            WAddressState => (361, 1),
            WAddressZip => (362, 1),
            WAddressCountry => (363, 1),
            WAddressGmtOffset => (364, 1),
            WNulls => (365, 2),
            WWarehouseAddress => (366, 7),
        }
    }
}

impl GeneratorColumn for WarehouseGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Warehouse
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
