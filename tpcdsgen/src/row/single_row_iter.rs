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
    use crate::row::{
        CallCenterRowGenerator, ItemRowGenerator, ReasonRowGenerator, StoreRowGenerator,
        WebPageRowGenerator, WebSiteRowGenerator,
    };

    fn session(scale_factor: f64) -> Session {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .build()
            .expect("session")
    }

    /// Collect the DAT text of the rows `G` emits for `table` over each of
    /// `ranges`, concatenated in order.
    fn rows_for<G: SingleRowGenerator>(
        generator: impl Fn() -> G,
        table: Table,
        session: &Session,
        ranges: &[(u64, u64)],
    ) -> Vec<String>
    where
        G::Row: std::fmt::Display,
    {
        let row_count = session.get_scaling().get_row_count(table);
        let mut out = Vec::new();
        for &(start, end) in ranges {
            let mut rows = SingleRowIter::new(generator(), session.clone(), row_count);
            rows.set_source_row_range(start, end);
            out.extend(rows.map(|row| row.to_string()));
        }
        out
    }

    /// Splitting a table into source row ranges must produce exactly the same
    /// rows as generating it in one pass.
    #[test]
    fn source_row_ranges_concatenate_to_the_unranged_output() {
        let session = session(1.0);
        let whole = rows_for(ReasonRowGenerator::new, Table::Reason, &session, &[(1, 35)]);
        let chunked = rows_for(
            ReasonRowGenerator::new,
            Table::Reason,
            &session,
            &[(1, 1), (2, 10), (11, 34), (35, 35)],
        );

        assert_eq!(whole.len(), 35);
        assert_eq!(whole, chunked);
    }

    /// An empty range produces nothing
    #[test]
    fn an_empty_range_produces_no_rows() {
        let session = session(1.0);
        let rows = rows_for(ReasonRowGenerator::new, Table::Reason, &session, &[(1, 0)]);
        assert!(rows.is_empty());
    }

    /// Assert that generating `table` one source row at a time reproduces the
    /// unranged output. Every row is a range start, so this covers each
    /// position of the six-row revision cycle.
    fn scd_single_row_ranges_match<G: SingleRowGenerator>(
        generator: impl Fn() -> G + Copy,
        table: Table,
    ) where
        G::Row: std::fmt::Display,
    {
        let session = session(1.0);
        // Two full revision cycles are enough; call_center only has six rows.
        let row_count = session.get_scaling().get_row_count(table).min(12);
        let singles: Vec<(u64, u64)> = (1..=row_count).map(|row| (row, row)).collect();

        let whole = rows_for(generator, table, &session, &[(1, row_count)]);
        assert_eq!(whole.len(), row_count as usize, "{table}");
        assert_eq!(
            whole,
            rows_for(generator, table, &session, &singles),
            "{table}"
        );
    }

    /// A range of an SCD table can start on a revision that copies values from
    /// the row before it, which the range never generates.
    #[test]
    fn scd_source_row_ranges_concatenate_to_the_unranged_output() {
        scd_single_row_ranges_match(ItemRowGenerator::new, Table::Item);
        scd_single_row_ranges_match(StoreRowGenerator::new, Table::Store);
        scd_single_row_ranges_match(WebPageRowGenerator::new, Table::WebPage);
        scd_single_row_ranges_match(WebSiteRowGenerator::new, Table::WebSite);
        scd_single_row_ranges_match(CallCenterRowGenerator::new, Table::CallCenter);
    }

    /// Reusing one generator across seeks must not carry revision state from
    /// the old position, including when seeking backwards or to the same row.
    #[test]
    fn seeking_an_scd_generator_rebuilds_its_history() {
        let session = session(1.0);
        let row_count = 12;
        let whole = rows_for(
            ItemRowGenerator::new,
            Table::Item,
            &session,
            &[(1, row_count)],
        );

        let mut rows = SingleRowIter::new(ItemRowGenerator::new(), session.clone(), row_count);
        for start in [row_count, 6, 3, 6, 1] {
            rows.skip_rows_until_starting_row_number(start);
            let row = rows.next().expect("row").to_string();
            assert_eq!(row, whole[start as usize - 1], "seek {start}");
        }
    }
}
