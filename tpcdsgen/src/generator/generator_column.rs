use crate::column::Table;

/// GeneratorColumns are columns that are used only within the context of the
/// generator logic. The Enums that implement this interface may include columns
/// that are not user visible and will sometimes omit columns that are user visible
/// (because those get derived from other columns).
///
/// GeneratorColumn
pub trait GeneratorColumn: Send + Sync {
    /// Get the table this generator column belongs to
    fn get_table(&self) -> Table;

    /// Get the global column number for this generator column
    fn get_global_column_number(&self) -> i32;

    /// Get the number of seeds per row for this generator column  
    fn get_seeds_per_row(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a simple test implementation
    struct TestGeneratorColumn {
        table: Table,
        global_column_number: i32,
        seeds_per_row: u32,
    }

    impl GeneratorColumn for TestGeneratorColumn {
        fn get_table(&self) -> Table {
            self.table
        }

        fn get_global_column_number(&self) -> i32 {
            self.global_column_number
        }

        fn get_seeds_per_row(&self) -> u32 {
            self.seeds_per_row
        }
    }

    #[test]
    fn test_generator_column_trait() {
        let test_column = TestGeneratorColumn {
            table: Table::CallCenter,
            global_column_number: 5,
            seeds_per_row: 10,
        };

        assert_eq!(test_column.get_table(), Table::CallCenter);
        assert_eq!(test_column.get_global_column_number(), 5);
        assert_eq!(test_column.get_seeds_per_row(), 10);
    }
}
