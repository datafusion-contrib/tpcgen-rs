use crate::generator::GeneratorColumn;
use crate::random::RandomNumberStream;
use crate::table::Table;

/// Abstract base for row generators (AbstractRowGenerator)
///
/// Manages the random number streams and walks the source rows
/// `current_row..=row_count` the generator is to produce.
pub struct AbstractRowGenerator {
    table: Table,
    /// The next source row to generate (1-based)
    current_row: u64,
    /// The last source row to generate (inclusive)
    row_count: u64,
    /// Random number streams stored in a Vec, indexed by (global_column_number - base_global_column_number)
    /// This replaces HashMap for O(1) direct array access without hashing overhead
    random_number_streams: Vec<RandomNumberStream>,
    /// The minimum global column number for this table's columns
    /// Used to convert global_column_number to Vec index
    base_global_column_number: i32,
}

impl AbstractRowGenerator {
    /// Create a new abstract row generator for source rows `1..=row_count`
    /// of the given table.
    ///
    /// Pre-creates all random number streams for the table's generator columns
    /// (matching Java's AbstractRowGenerator constructor behavior)
    pub fn new(table: Table, row_count: u64) -> Self {
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
                    RandomNumberStream::new_with_column(global_column_number, seeds_per_row)
                        .expect("Failed to create random number stream");
                random_number_streams.push(stream);
            }
        }

        Self {
            table,
            current_row: 1,
            row_count,
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
    ) -> &mut RandomNumberStream {
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

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// all streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        let rows_to_skip = starting_row_number.saturating_sub(1);
        for stream in self.random_number_streams.iter_mut() {
            stream.skip_rows(rows_to_skip);
        }
        self.current_row = starting_row_number;
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.skip_rows_until_starting_row_number(starting_row_number);
        self.row_count = self.row_count.min(ending_row_number);
    }

    /// The source row to generate next, or `None` when past the end of the
    /// range.
    ///
    /// Call [`Self::finish_row`] once the row is generated to move on.
    pub fn next_row_number(&self) -> Option<u64> {
        (self.current_row <= self.row_count).then_some(self.current_row)
    }

    /// Consume the seeds the current row left unused and move to the next
    /// row.
    pub fn finish_row(&mut self) {
        self.consume_remaining_seeds_for_row();
        self.current_row += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::CallCenterGeneratorColumn;

    #[test]
    fn test_abstract_row_generator_creation() {
        let generator = AbstractRowGenerator::new(Table::CallCenter, 6);
        assert_eq!(generator.get_table(), Table::CallCenter);
    }

    #[test]
    fn test_random_number_stream_creation() {
        let mut generator = AbstractRowGenerator::new(Table::CallCenter, 6);
        let column = &CallCenterGeneratorColumn::CcCallCenterSk;

        let _stream1 = generator.get_random_number_stream(column);
        let _stream2 = generator.get_random_number_stream(column);

        // Should reuse the same stream for the same column
        assert_eq!(generator.random_number_streams.len(), 34);
    }

    #[test]
    fn walks_the_source_row_range() {
        let mut generator = AbstractRowGenerator::new(Table::CallCenter, 6);
        assert_eq!(generator.next_row_number(), Some(1));
        generator.finish_row();
        assert_eq!(generator.next_row_number(), Some(2));

        generator.set_source_row_range(5, 100);
        assert_eq!(generator.next_row_number(), Some(5));
        generator.finish_row();
        assert_eq!(generator.next_row_number(), Some(6));
        generator.finish_row();
        assert_eq!(generator.next_row_number(), None);

        generator.skip_rows_until_starting_row_number(3);
        assert_eq!(generator.next_row_number(), Some(3));
    }

    #[test]
    fn test_multiple_column_streams() {
        let mut generator = AbstractRowGenerator::new(Table::CallCenter, 6);
        let col1 = &CallCenterGeneratorColumn::CcCallCenterSk;
        let col2 = &CallCenterGeneratorColumn::CcCallCenterId;

        let _stream1 = generator.get_random_number_stream(col1);
        let _stream2 = generator.get_random_number_stream(col2);

        // Should create separate streams for different columns
        assert_eq!(generator.random_number_streams.len(), 34);
    }
}
