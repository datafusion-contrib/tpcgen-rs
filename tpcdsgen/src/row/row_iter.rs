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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SessionBuilder, Table};
    use crate::row::{
        CallCenterRowGenerator, ItemRowGenerator, ReasonRowGenerator, StoreRowGenerator,
        StoreSalesRowGenerator, WebPageRowGenerator, WebSiteRowGenerator,
    };

    fn session(scale_factor: f64) -> Session {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .build()
            .expect("session")
    }

    /// Collect the DAT text of the rows `G` emits for `table` over each of
    /// `ranges`, concatenated in order.
    fn rows_for<G: RowGenerator>(
        generator: impl Fn() -> G,
        table: Table,
        session: &Session,
        ranges: &[(u64, u64)],
    ) -> Vec<String> {
        let row_count = session.get_scaling().get_row_count(table.source_table());
        let mut out = Vec::new();
        for &(start, end) in ranges {
            let mut rows = RowIter::new(generator(), session.clone(), row_count);
            rows.set_source_row_range(start, end);
            out.extend(
                rows.filter(|row| row.table() == table)
                    .map(|row| row.to_string()),
            );
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

    /// The sales generators emit rows for two tables, so a caller that wants
    /// only one of them filters on [`GeneratedRow::table`].
    #[test]
    fn a_sales_generator_emits_both_of_its_tables_over_a_range() {
        let session = session(0.01);
        let source_rows = session.get_scaling().get_row_count(Table::StoreSales);
        assert!(source_rows > 100, "need enough rows to split");
        let split = [(1, source_rows / 2), (source_rows / 2 + 1, source_rows)];

        for table in [Table::StoreSales, Table::StoreReturns] {
            let whole = rows_for(
                StoreSalesRowGenerator::new,
                table,
                &session,
                &[(1, source_rows)],
            );
            let chunked = rows_for(StoreSalesRowGenerator::new, table, &session, &split);

            assert!(!whole.is_empty(), "{table} produced no rows");
            assert_eq!(whole, chunked, "{table} ranged output differs");
        }
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
    fn scd_single_row_ranges_match<G: RowGenerator>(
        generator: impl Fn() -> G + Copy,
        table: Table,
    ) {
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

        let mut rows = RowIter::new(ItemRowGenerator::new(), session.clone(), row_count);
        for start in [row_count, 6, 3, 6, 1] {
            rows.skip_rows_until_starting_row_number(start);
            let row = rows.next().expect("row").to_string();
            assert_eq!(row, whole[start as usize - 1], "seek {start}");
        }
    }
}
