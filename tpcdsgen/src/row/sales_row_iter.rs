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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SessionBuilder, Table};
    use crate::row::StoreSalesRowGenerator;

    /// Collect the DAT text of the rows `G` emits over each of `ranges`,
    /// concatenated in order. `select` picks the sales or returns row out of
    /// each [`SalesRows`].
    fn rows_for<G, R>(
        generator: impl Fn() -> G,
        select: impl Fn(SalesRows<G::Sales, G::Returns>) -> Option<R>,
        session: &Session,
        row_count: u64,
        ranges: &[(u64, u64)],
    ) -> Vec<String>
    where
        G: SalesRowGenerator,
        R: std::fmt::Display,
    {
        let mut out = Vec::new();
        for &(start, end) in ranges {
            let mut rows = SalesRowIter::new(generator(), session.clone(), row_count);
            rows.set_source_row_range(start, end);
            out.extend(rows.filter_map(&select).map(|row| row.to_string()));
        }
        out
    }

    /// Splitting a table into source row ranges must produce exactly the same
    /// rows as generating it in one pass, for both the sales and the returns
    /// selection of a sales generator.
    #[test]
    fn source_row_ranges_concatenate_to_the_unranged_output() {
        let session = SessionBuilder::new()
            .with_scale_factor(0.01)
            .build()
            .expect("session");
        let source_rows = session.get_scaling().get_row_count(Table::StoreSales);
        assert!(source_rows > 100, "need enough rows to split");
        let split = [(1, source_rows / 2), (source_rows / 2 + 1, source_rows)];

        let sales = |rows: SalesRows<_, _>| rows.sales;
        let whole = rows_for(
            StoreSalesRowGenerator::sales,
            sales,
            &session,
            source_rows,
            &[(1, source_rows)],
        );
        let chunked = rows_for(
            StoreSalesRowGenerator::sales,
            sales,
            &session,
            source_rows,
            &split,
        );
        assert!(!whole.is_empty(), "store_sales produced no rows");
        assert_eq!(whole, chunked, "store_sales ranged output differs");

        let returns = |rows: SalesRows<_, _>| rows.returns;
        let whole = rows_for(
            StoreSalesRowGenerator::returns,
            returns,
            &session,
            source_rows,
            &[(1, source_rows)],
        );
        let chunked = rows_for(
            StoreSalesRowGenerator::returns,
            returns,
            &session,
            source_rows,
            &split,
        );
        assert!(!whole.is_empty(), "store_returns produced no rows");
        assert_eq!(whole, chunked, "store_returns ranged output differs");
    }
}
