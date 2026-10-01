/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! [`SingleRowIter`]: stream concrete rows from a [`SingleRowGenerator`].

use crate::config::Session;
use crate::row::SingleRowGenerator;

/// Adapts a [`SingleRowGenerator`] into an [`Iterator`] of its concrete `Row`
/// type.
///
/// It is possible to restrict the iterator to a range of source rows with
/// [`Self::set_source_row_range`].
///
/// [`GeneratedRow`]: crate::row::GeneratedRow
pub struct SingleRowIter<G: SingleRowGenerator> {
    generator: G,
    session: Session,
    current_row: u64,
    row_count: u64,
}

impl<G: SingleRowGenerator> SingleRowIter<G> {
    /// Generate source rows `1..=row_count`.
    pub fn new(generator: G, session: Session, row_count: u64) -> Self {
        Self {
            generator,
            session,
            current_row: 1,
            row_count,
        }
    }

    /// Start generating at `starting_row_number` (1-based), fast forwarding
    /// the generator's random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.generator
            .skip_rows_until_starting_row_number(starting_row_number);
        self.current_row = starting_row_number;
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


impl<G: SingleRowGenerator> Iterator for SingleRowIter<G> {
    type Item = G::Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row > self.row_count {
            return None;
        }
        let row = self
            .generator
            .generate_row(self.current_row, &self.session)
            .expect("row gen");
        self.generator.consume_remaining_seeds_for_row();
        self.current_row += 1;
        Some(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SessionBuilder, Table};
    use crate::row::InventoryRowGenerator;

    fn session(scale_factor: f64) -> Session {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .build()
            .expect("session")
    }

    /// Collect the DAT text of the rows `InventoryRowGenerator` emits over
    /// each of `ranges`, concatenated in order.
    fn rows_for(session: &Session, ranges: &[(u64, u64)]) -> Vec<String> {
        let row_count = session.get_scaling().get_row_count(Table::Inventory);
        let mut out = Vec::new();
        for &(start, end) in ranges {
            let mut rows =
                SingleRowIter::new(InventoryRowGenerator::new(), session.clone(), row_count);
            rows.set_source_row_range(start, end);
            out.extend(rows.map(|row| row.to_string()));
        }
        out
    }

    /// Splitting the table into source row ranges must produce exactly the
    /// same rows as generating it in one pass.
    #[test]
    fn source_row_ranges_concatenate_to_the_unranged_output() {
        let session = session(0.01);
        let row_count = session.get_scaling().get_row_count(Table::Inventory);
        assert!(row_count > 100, "need enough rows to split");
        let split = [(1, row_count / 2), (row_count / 2 + 1, row_count)];

        let whole = rows_for(&session, &[(1, row_count)]);
        let chunked = rows_for(&session, &split);

        assert!(!whole.is_empty());
        assert_eq!(whole, chunked);
    }

    /// An empty range produces nothing
    #[test]
    fn an_empty_range_produces_no_rows() {
        let session = session(0.01);
        let rows = rows_for(&session, &[(1, 0)]);
        assert!(rows.is_empty());
    }

    /// Reusing one generator across seeks must not depend on the prior
    /// position.
    #[test]
    fn seeking_rebuilds_from_the_new_position() {
        let session = session(0.01);
        let row_count = session.get_scaling().get_row_count(Table::Inventory);
        let whole = rows_for(&session, &[(1, row_count)]);

        let mut rows = SingleRowIter::new(InventoryRowGenerator::new(), session.clone(), row_count);
        for start in [row_count, row_count / 2, 1] {
            rows.skip_rows_until_starting_row_number(start);
            let row = rows.next().expect("row").to_string();
            assert_eq!(row, whole[start as usize - 1], "seek {start}");
        }
    }
}
