use crate::column::Table;
use crate::generator::GeneratorColumn;

/// Reason generator columns (ReasonGeneratorColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasonGeneratorColumn {
    RReasonSk,
    RReasonId,
    RReasonDescription,
    RNulls,
}

impl ReasonGeneratorColumn {
    /// Get all generator columns in order
    pub fn values() -> &'static [ReasonGeneratorColumn] {
        use ReasonGeneratorColumn::*;
        static VALUES: &[ReasonGeneratorColumn] =
            &[RReasonSk, RReasonId, RReasonDescription, RNulls];
        VALUES
    }

    /// Get the global column number and seeds per row for this generator column
    /// Values exactly match Java implementation
    fn get_column_info(&self) -> (i32, u32) {
        use ReasonGeneratorColumn::*;
        match self {
            RReasonSk => (248, 1),
            RReasonId => (249, 1),
            RReasonDescription => (250, 1),
            RNulls => (251, 2),
        }
    }
}

impl GeneratorColumn for ReasonGeneratorColumn {
    fn get_table(&self) -> Table {
        Table::Reason
    }

    fn get_global_column_number(&self) -> i32 {
        self.get_column_info().0
    }

    fn get_seeds_per_row(&self) -> u32 {
        self.get_column_info().1
    }
}
