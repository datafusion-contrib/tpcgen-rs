use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Ship mode generator columns (ShipModeGeneratorColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShipModeGeneratorColumn {
    SmShipModeSk,
    SmShipModeId,
    SmType,
    SmCode,
    SmContract,
    SmCarrier,
    SmNulls,
}

impl ShipModeGeneratorColumn {
    /// Get all generator columns in order
    pub fn values() -> &'static [ShipModeGeneratorColumn] {
        use ShipModeGeneratorColumn::*;
        static VALUES: &[ShipModeGeneratorColumn] = &[
            SmShipModeSk,
            SmShipModeId,
            SmType,
            SmCode,
            SmContract,
            SmCarrier,
            SmNulls,
        ];
        VALUES
    }

    /// Get the global column number and seeds per row for this generator column
    /// Values exactly match Java implementation
    fn get_column_info(&self) -> (i32, u32) {
        use ShipModeGeneratorColumn::*;
        match self {
            SmShipModeSk => (252, 1),
            SmShipModeId => (253, 1),
            SmType => (254, 1),
            SmCode => (255, 1),
            SmContract => (256, 21),
            SmCarrier => (257, 1),
            SmNulls => (258, 2),
        }
    }
}

impl GeneratorColumn for ShipModeGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::ShipMode
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
