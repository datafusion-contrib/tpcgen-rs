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

//! [`SalesRowIter`]: stream concrete rows from a [`SalesRowGenerator`].

use crate::config::Session;
use crate::row::{SalesRowGenerator, SalesRows};

/// Adapts a [`SalesRowGenerator`] into an [`Iterator`] of its concrete
/// [`SalesRows`] type, one per line item.
///
/// Callers wanting only the sales rows map to `.sales`; callers wanting only
/// the returns rows filter on `.returns`.
///
/// It is possible to restrict the iterator to a range of source rows with
/// [`Self::set_source_row_range`].
pub struct SalesRowIter<G: SalesRowGenerator> {
    generator: G,
    session: Session,
    current_row: u64,
    row_count: u64,
}

impl<G: SalesRowGenerator> SalesRowIter<G> {
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

impl<G: SalesRowGenerator> Iterator for SalesRowIter<G> {
    type Item = SalesRows<G::Sales, G::Returns>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row > self.row_count {
            return None;
        }
        let rows = self
            .generator
            .generate_row(self.current_row, &self.session)
            .expect("row gen");
        if self.generator.is_last_row_in_order() {
            self.generator.consume_remaining_seeds_for_row();
            self.current_row += 1;
        }
        Some(rows)
    }
}
