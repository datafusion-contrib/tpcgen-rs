//! Progress registration shared by the TPC-DS row-generator outputs.
//!
//! The DAT and CSV outputs both drive the row generators directly and pair
//! sales tables with their returns table, so they register progress the same
//! way. Keeping that in one place stops the two from drifting.

use crate::progress::{ProgressHandle, ProgressTracker};
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};

/// Progress handles for one requested table.
///
/// Sales tables are generated together with their returns table, so they
/// register two handles; the returns tables themselves register none.
#[derive(Debug, Clone)]
pub(super) enum TableProgress {
    None,
    Single(ProgressHandle),
    Paired {
        sales: ProgressHandle,
        returns: ProgressHandle,
    },
}

/// Number of rows in a source row range, treating an empty chunk as zero.
fn range_len(range: RangeInclusive<u64>) -> u64 {
    if range.is_empty() {
        0
    } else {
        range.end() - range.start() + 1
    }
}

/// Register progress for one requested table, sized to the rows `sessions`
/// will actually generate.
///
/// `sessions` holds one session per requested part. Callers register once, then
/// split the result with [`share_across_parts`].
///
/// A returns table's row count is only ever an approximate upper bound (actual
/// returns are data-driven per source row), and is counted in returned rows
/// rather than source rows
pub(super) fn register_table(
    table: Table,
    sessions: &[Session],
    progress: Arc<dyn ProgressTracker>,
) -> TableProgress {
    let scaling = sessions[0].get_scaling();
    let requested_source_rows: u64 = sessions
        .iter()
        .map(|session| range_len(session.get_source_row_range(table)))
        .sum();
    let all_source_rows = scaling.get_row_count(table.source_table());

    let register = |registered: Table| {
        let total = if registered == table {
            requested_source_rows
        } else if all_source_rows == 0 {
            0
        } else {
            // u128 so the numerator cannot overflow at large scale factors.
            (scaling.get_row_count(registered) as u128 * requested_source_rows as u128
                / all_source_rows as u128) as u64
        };
        progress.clone().register(registered.get_name(), total)
    };

    match table {
        Table::StoreSales => TableProgress::Paired {
            sales: register(Table::StoreSales),
            returns: register(Table::StoreReturns),
        },
        Table::CatalogSales => TableProgress::Paired {
            sales: register(Table::CatalogSales),
            returns: register(Table::CatalogReturns),
        },
        Table::WebSales => TableProgress::Paired {
            sales: register(Table::WebSales),
            returns: register(Table::WebReturns),
        },
        Table::StoreReturns | Table::CatalogReturns | Table::WebReturns => TableProgress::None,
        _ => TableProgress::Single(register(table)),
    }
}

/// Split one registered handle into `num_parts` clones that all report to the
/// same bar.
///
/// Each part finishes independently and calls [`ProgressHandle::complete`] on
/// its own, so we need to ensure the bar only reaches its "done" style once
/// every part has completed.
pub(super) fn share_handle_across_parts(
    handle: ProgressHandle,
    num_parts: usize,
) -> Vec<ProgressHandle> {
    let remaining = Arc::new(AtomicUsize::new(num_parts.max(1)));
    (0..num_parts.max(1))
        .map(|_| {
            let remaining = remaining.clone();
            let increment_handle = handle.clone();
            let complete_handle = handle.clone();
            ProgressHandle::new_with_complete(
                move |units| increment_handle.increment(units),
                move || {
                    if remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
                        complete_handle.complete();
                    }
                },
            )
        })
        .collect()
}

/// [`share_handle_across_parts`] applied to every handle a [`TableProgress`]
/// holds, so a paired sales/returns table shares each half independently.
pub(super) fn share_across_parts(progress: TableProgress, num_parts: usize) -> Vec<TableProgress> {
    match progress {
        TableProgress::None => (0..num_parts.max(1)).map(|_| TableProgress::None).collect(),
        TableProgress::Single(handle) => share_handle_across_parts(handle, num_parts)
            .into_iter()
            .map(TableProgress::Single)
            .collect(),
        TableProgress::Paired { sales, returns } => share_handle_across_parts(sales, num_parts)
            .into_iter()
            .zip(share_handle_across_parts(returns, num_parts))
            .map(|(sales, returns)| TableProgress::Paired { sales, returns })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    use std::sync::Mutex;
    use tpcdsgen::config::SessionBuilder;

    #[derive(Debug, Default)]
    struct RecordingProgress {
        registered: Mutex<Vec<(String, u64)>>,
    }

    impl ProgressTracker for RecordingProgress {
        fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle {
            self.registered
                .lock()
                .unwrap()
                .push((item.to_owned(), total_units));
            ProgressHandle::new(|_| {})
        }
    }

    /// A handle plus counters for what it observed, for asserting on
    /// [`share_handle_across_parts`]'s forwarding behavior.
    fn recording_handle() -> (ProgressHandle, Arc<AtomicU64>, Arc<AtomicUsize>) {
        let increments = Arc::new(AtomicU64::new(0));
        let completions = Arc::new(AtomicUsize::new(0));
        let handle = {
            let increments = increments.clone();
            let completions = completions.clone();
            ProgressHandle::new_with_complete(
                move |units| {
                    increments.fetch_add(units, Ordering::Relaxed);
                },
                move || {
                    completions.fetch_add(1, Ordering::Relaxed);
                },
            )
        };
        (handle, increments, completions)
    }

    /// Sessions for every chunk of a `--parts total_chunks` run.
    fn all_parts(scale_factor: f64, total_chunks: i32) -> Vec<Session> {
        (1..=total_chunks)
            .map(|chunk_number| chunk(scale_factor, chunk_number, total_chunks))
            .collect()
    }

    /// A session for one `--parts total_chunks --part chunk_number` run.
    fn chunk(scale_factor: f64, chunk_number: i32, total_chunks: i32) -> Session {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .with_chunk_number(chunk_number)
            .with_total_chunks(total_chunks)
            .with_partitioned(true)
            .build()
            .unwrap()
    }

    #[test]
    fn register_table_total_covers_every_requested_part() {
        // `--parts 4` runs all four chunks in this process, so the shared bar
        // is sized to the whole table, exactly as an unpartitioned run is.
        let tracker = Arc::new(RecordingProgress::default());
        let whole = SessionBuilder::new()
            .with_scale_factor(5.0)
            .build()
            .unwrap();

        register_table(Table::StoreSales, &[whole], tracker.clone());
        register_table(Table::StoreSales, &all_parts(5.0, 4), tracker.clone());

        let registered = tracker.registered.lock().unwrap();
        assert_eq!(registered[0..2], registered[2..4]);
    }

    #[test]
    fn register_table_total_is_sized_to_a_single_requested_part() {
        // `--parts 4 --part 2` generates only chunk 2, so a bar sized to the
        // whole table would top out at a quarter and then snap to "done".
        let tracker = Arc::new(RecordingProgress::default());
        let whole = SessionBuilder::new()
            .with_scale_factor(7.0)
            .build()
            .unwrap();
        let one_of_four = chunk(7.0, 2, 4);
        let all_rows = range_len(whole.get_source_row_range(Table::CatalogSales));
        let part_rows = range_len(one_of_four.get_source_row_range(Table::CatalogSales));
        assert!(
            part_rows > 0 && part_rows < all_rows,
            "chunk must be a strict subset"
        );

        register_table(Table::CatalogSales, &[whole], tracker.clone());
        register_table(Table::CatalogSales, &[one_of_four], tracker.clone());

        let registered = tracker.registered.lock().unwrap();
        assert_eq!(registered[0], ("catalog_sales".to_owned(), all_rows));
        assert_eq!(registered[2], ("catalog_sales".to_owned(), part_rows));
        // The returns bar counts returned rows, not source rows, so its
        // (approximate) whole-table total is scaled to this chunk's share.
        let whole_returns = registered[1].1;
        assert_eq!(
            registered[3],
            (
                "catalog_returns".to_owned(),
                whole_returns * part_rows / all_rows
            )
        );
    }

    #[test]
    fn register_table_total_is_zero_for_an_empty_chunk() {
        // `reason` is far below the 1M threshold, so every chunk past the
        // first is empty and its bar has nothing to report.
        let tracker = Arc::new(RecordingProgress::default());

        register_table(Table::Reason, &[chunk(1.0, 2, 4)], tracker.clone());

        let registered = tracker.registered.lock().unwrap();
        assert_eq!(registered[0].1, 0);
    }

    #[test]
    fn share_handle_across_parts_forwards_every_increment() {
        let (handle, increments, _completions) = recording_handle();
        let parts = share_handle_across_parts(handle, 4);
        assert_eq!(parts.len(), 4);

        for part in &parts {
            part.increment(1);
        }
        assert_eq!(increments.load(Ordering::Relaxed), 4);
    }

    #[test]
    fn share_handle_across_parts_completes_only_after_every_part_completes() {
        let (handle, _increments, completions) = recording_handle();
        let parts = share_handle_across_parts(handle, 3);

        parts[0].complete();
        parts[1].complete();
        assert_eq!(
            completions.load(Ordering::Relaxed),
            0,
            "must not finish the bar before the last part reports completion"
        );

        parts[2].complete();
        assert_eq!(completions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn share_across_parts_gates_sales_and_returns_independently() {
        let (sales_handle, _sales_inc, sales_completions) = recording_handle();
        let (returns_handle, _returns_inc, returns_completions) = recording_handle();
        let progress = TableProgress::Paired {
            sales: sales_handle,
            returns: returns_handle,
        };

        let mut parts = share_across_parts(progress, 2).into_iter();
        let (
            TableProgress::Paired {
                sales: sales1,
                returns: returns1,
            },
            TableProgress::Paired {
                sales: sales2,
                returns: returns2,
            },
        ) = (parts.next().unwrap(), parts.next().unwrap())
        else {
            panic!("expected two paired parts");
        };

        sales1.complete();
        returns1.complete();
        assert_eq!(sales_completions.load(Ordering::Relaxed), 0);
        assert_eq!(returns_completions.load(Ordering::Relaxed), 0);

        sales2.complete();
        assert_eq!(sales_completions.load(Ordering::Relaxed), 1);
        assert_eq!(
            returns_completions.load(Ordering::Relaxed),
            0,
            "sales and returns gates are independent"
        );

        returns2.complete();
        assert_eq!(returns_completions.load(Ordering::Relaxed), 1);
    }
}
