use crate::generator::GeneratorColumn;
use crate::random::{RandomNumberStream, RandomNumberStreamImpl};
use crate::table::Table;

/// Abstract base for row generators (AbstractRowGenerator)
/// Handles common functionality like random number stream management
pub struct AbstractRowGenerator {
    table: Table,
    /// Random number streams stored in a Vec, indexed by (global_column_number - base_global_column_number)
    /// This replaces HashMap for O(1) direct array access without hashing overhead
    random_number_streams: Vec<RandomNumberStreamImpl>,
    /// The minimum global column number for this table's columns
    /// Used to convert global_column_number to Vec index
    base_global_column_number: i32,
}

impl AbstractRowGenerator {
    /// Create a new abstract row generator for the given table
    /// Pre-creates all random number streams for the table's generator columns
    /// (matching Java's AbstractRowGenerator constructor behavior)
    pub fn new(table: Table) -> Self {
        let column_count = table.get_generator_column_count();

        // Find the base global column number (minimum among all columns)
        let base_global_column_number = if column_count > 0 {
            table
                .get_generator_column_by_index(0)
                .map(|col| col.get_global_column_number())
                .unwrap_or(0)
        } else {
            0
        };

        // Pre-create all streams for this table's generator columns
        // This is critical because consume_remaining_seeds_for_row needs to advance
        // ALL streams, even ones that haven't been accessed yet
        let mut random_number_streams = Vec::with_capacity(column_count);

        for i in 0..column_count {
            if let Some(gen_col) = table.get_generator_column_by_index(i) {
                let global_column_number = gen_col.get_global_column_number();
                let seeds_per_row = gen_col.get_seeds_per_row();

                let stream =
                    RandomNumberStreamImpl::new_with_column(global_column_number, seeds_per_row)
                        .expect("Failed to create random number stream");
                random_number_streams.push(stream);
            }
        }

        Self {
            table,
            random_number_streams,
            base_global_column_number,
        }
    }

    /// Get the table this generator is for
    pub fn get_table(&self) -> Table {
        self.table
    }

    /// Get a random number stream for a generator column
    /// Uses direct array indexing for O(1) access
    pub fn get_random_number_stream(
        &mut self,
        column: &dyn GeneratorColumn,
    ) -> &mut dyn RandomNumberStream {
        let global_column_number = column.get_global_column_number();
        let index = (global_column_number - self.base_global_column_number) as usize;

        &mut self.random_number_streams[index]
    }

    /// Consume remaining seeds for all streams (AbstractRowGenerator.consumeRemainingSeedsForRow)
    pub fn consume_remaining_seeds_for_row(&mut self) {
        use crate::random::RandomValueGenerator;

        for stream in self.random_number_streams.iter_mut() {
            // Consume remaining seeds until each stream has used its full seeds_per_row allocation
            while stream.get_seeds_used() < stream.get_seeds_per_row() {
                RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
            }
            // Reset seeds used count for next row
            stream.reset_seeds_used();
        }
    }

    /// Skip rows for all streams until reaching the starting row number
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        let rows_to_skip = starting_row_number.saturating_sub(1);
        for stream in self.random_number_streams.iter_mut() {
            stream.skip_rows(rows_to_skip);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::CallCenterGeneratorColumn;

    #[test]
    fn test_abstract_row_generator_creation() {
        let generator = AbstractRowGenerator::new(Table::CallCenter);
        assert_eq!(generator.get_table(), Table::CallCenter);
    }

    #[test]
    fn test_random_number_stream_creation() {
        let mut generator = AbstractRowGenerator::new(Table::CallCenter);
        let column = &CallCenterGeneratorColumn::CcCallCenterSk;

        let _stream1 = generator.get_random_number_stream(column);
        let _stream2 = generator.get_random_number_stream(column);

        // Should reuse the same stream for the same column
        assert_eq!(generator.random_number_streams.len(), 34);
    }

    #[test]
    fn test_multiple_column_streams() {
        let mut generator = AbstractRowGenerator::new(Table::CallCenter);
        let col1 = &CallCenterGeneratorColumn::CcCallCenterSk;
        let col2 = &CallCenterGeneratorColumn::CcCallCenterId;

        let _stream1 = generator.get_random_number_stream(col1);
        let _stream2 = generator.get_random_number_stream(col2);

        // Should create separate streams for different columns
        assert_eq!(generator.random_number_streams.len(), 34);
    }
}
