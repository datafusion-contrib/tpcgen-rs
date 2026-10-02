//! [`RowIter`]: stream [`GeneratedRow`]s from a [`RowGenerator`].

use crate::config::Session;
use crate::row::{GeneratedRow, RowGenerator};
use std::collections::VecDeque;

/// Adapts a [`RowGenerator`] into a streaming [`Iterator`] of [`GeneratedRow`]s.
///
/// # Simple generators vs Paired fact-table generators
///
/// Simple generators make one row per call and
/// [`RowGeneratorResult::should_end_row`] returns true.
///
/// Paired fact-table generators return multiple rows per source row, and
/// [`RowGeneratorResult::should_end_row`] signals when to advance the row
/// counter. For example, `store_sales` also generates rows for `store_returns`.
/// Use [`GeneratedRow::table`] to filter the output to a single table if desired.
///
/// It is also possible to restrict the iterator to a range of source rows with
/// [`Self::set_source_row_range`].
///
/// [`RowGeneratorResult::should_end_row`]: crate::row::RowGeneratorResult::should_end_row
pub struct RowIter<G: RowGenerator> {
    generator: G,
    session: Session,
    current_row: u64,
    row_count: u64,
    pending: VecDeque<GeneratedRow>,
}

impl<G: RowGenerator> RowIter<G> {
    /// Generate source rows `1..=row_count`.
    pub fn new(generator: G, session: Session, row_count: u64) -> Self {
        Self {
            generator,
            session,
            current_row: 1,
            row_count,
            pending: VecDeque::new(),
        }
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the generator's random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.current_row = starting_row_number;
        self.pending.clear();
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.skip_rows_until_starting_row_number(starting_row_number);
        self.row_count = self.row_count.min(ending_row_number);
    }
}

impl<G: RowGenerator> Iterator for RowIter<G> {
    type Item = GeneratedRow;

    fn next(&mut self) -> Option<GeneratedRow> {
        while self.pending.is_empty() {
            if self.current_row > self.row_count {
                return None;
            }
            let result = self
                .generator
                .generate_row_and_child_rows(self.current_row, &self.session, None, None)
                .expect("row gen");
            let (rows, should_end_row) = result.into_parts();
            self.pending.extend(rows);
            if should_end_row {
                self.generator.consume_remaining_seeds_for_row();
                self.current_row += 1;
            }
        }
        self.pending.pop_front()
    }
}
