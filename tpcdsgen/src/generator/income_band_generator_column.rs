use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Income band generator columns (IncomeBandGeneratorColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IncomeBandGeneratorColumn {
    IbIncomeBandId,
    IbLowerBound,
    IbUpperBound,
    IbNulls,
}

impl IncomeBandGeneratorColumn {
    /// Get all generator columns in order
    pub fn values() -> &'static [IncomeBandGeneratorColumn] {
        use IncomeBandGeneratorColumn::*;
        static VALUES: &[IncomeBandGeneratorColumn] =
            &[IbIncomeBandId, IbLowerBound, IbUpperBound, IbNulls];
        VALUES
    }

    /// Get the global column number and seeds per row for this generator column
    /// Values exactly match Java implementation
    fn get_column_info(&self) -> (i32, u32) {
        use IncomeBandGeneratorColumn::*;
        match self {
            IbIncomeBandId => (194, 1),
            IbLowerBound => (195, 1),
            IbUpperBound => (196, 1),
            IbNulls => (197, 2),
        }
    }
}

impl GeneratorColumn for IncomeBandGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::IncomeBand
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
