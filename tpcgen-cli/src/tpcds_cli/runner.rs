//! Planning and scheduling TPC-DS table generation, shared by every output.
//!
//! All three TPC-DS outputs work the same way, and the same way the TPC-H
//! outputs do (see [`crate::tpch_cli::runner`]): split each requested table
//! into chunks of source rows, generate the chunks in parallel, and have a
//! single writer append them to the output file in order. Only the per chunk
//! formatting differs between DAT, CSV and Parquet, so the planning, the
//! progress registration and the scheduling live here.

use super::plan::{ChunkFormat, TpcdsGenerationPlan};
use super::progress::share_handle_across_parts;
use crate::progress::{ProgressHandle, ProgressTracker};
use crate::worker_queue::WorkerQueue;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};

/// One unit of schedulable work: the chunks of one table for one `--parts`
/// chunk, together with the progress handle they report to.
#[derive(Debug)]
pub(super) struct PlannedTable {
    /// The table to write
    pub(super) table: Table,
    /// The session describing the scale factor, compat mode and `--parts`
    /// chunk this output covers
    pub(super) session: Session,
    /// How this output's source rows are split into chunks
    pub(super) plan: TpcdsGenerationPlan,
    /// Advanced once per written chunk
    pub(super) progress: ProgressHandle,
}

/// Plan every requested `(table, session)` and register the progress bars.
///
/// A table split across `--parts` gets one bar for all of its parts combined,
/// not one bar per part. Empty `--parts` chunks are dropped: dsdgen generates
/// a table below its 1M source row threshold entirely in chunk 1, so the
/// later chunks have no rows and no file to write.
pub(super) fn plan_tables(
    table_sessions: Vec<(Table, Session)>,
    chunk_bytes: i64,
    format: ChunkFormat,
    progress: &Arc<dyn ProgressTracker>,
) -> Vec<PlannedTable> {
    // Group all sessions that contribute to the same table progress bar.
    let mut sessions_by_table: HashMap<Table, Vec<Session>> = HashMap::new();
    for (table, session) in table_sessions {
        sessions_by_table.entry(table).or_default().push(session);
    }

    // Prepare each table before scheduling: plan its nonempty sessions and
    // register one shared progress bar sized to their combined chunks.
    let mut prepared = Vec::new();
    for (table, sessions) in sessions_by_table {
        let planned: Vec<(Session, TpcdsGenerationPlan)> = sessions
            .into_iter()
            .filter_map(|session| {
                let row_range = session.get_source_row_range(table);
                if row_range.is_empty() && session.is_partitioned() {
                    return None;
                }
                let plan =
                    TpcdsGenerationPlan::new_for_range(table, chunk_bytes, row_range, format);
                Some((session, plan))
            })
            .collect();

        if planned.is_empty() {
            continue;
        }

        let total_chunks = planned
            .iter()
            .map(|(_, plan)| plan.chunk_count() as u64)
            .sum();
        let table_progress = progress.clone().register(table.get_name(), total_chunks);
        prepared.push((table, planned, table_progress));
    }

    // Fan the table progress out so every session reports to the same bar,
    // then pair each session with its progress to build schedulable work.
    let mut work = Vec::new();
    for (table, planned, table_progress) in prepared {
        let part_progress = share_handle_across_parts(table_progress, planned.len());
        for ((session, plan), progress) in planned.into_iter().zip(part_progress) {
            work.push(PlannedTable {
                table,
                session,
                plan,
                progress,
            });
        }
    }
    work
}

/// Generate every [`PlannedTable`] by calling `generate`, running within an
/// overall budget of `num_threads` threads.
///
/// Tables are generated concurrently: each table's plan gets as many threads
/// as it has chunks, within the overall budget (see [`WorkerQueue`]).
/// Scheduling the largest tables first keeps all cores busy while the trailing
/// chunks of each table are written, instead of waiting for one table at a
/// time.
pub(super) async fn run_plans<F, Fut>(
    mut work: Vec<PlannedTable>,
    num_threads: usize,
    generate: F,
) -> io::Result<()>
where
    F: Fn(PlannedTable, usize) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = io::Result<()>> + Send + 'static,
{
    // Schedule the largest tables (most chunks) first for the best thread
    // utilization (the list is popped from the back)
    work.sort_by_key(|planned| planned.plan.chunk_count());

    let generate = Arc::new(generate);
    let mut queue = WorkerQueue::new(num_threads);
    while let Some(planned) = work.pop() {
        let chunk_count = planned.plan.chunk_count();
        let generate = Arc::clone(&generate);
        queue
            .schedule(chunk_count, move |num_threads| async move {
                generate(planned, num_threads).await?;
                Ok(num_threads)
            })
            .await?;
    }
    queue.join_all().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
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

    fn whole(scale_factor: f64) -> Session {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .build()
            .unwrap()
    }

    const CHUNK_BYTES: i64 = 7 * 1024 * 1024;

    /// dsdgen generates a table below 1M source rows entirely in chunk 1, so
    /// splitting across parts only does anything above that threshold.
    const PARTITIONABLE_SCALE: f64 = 10.0;

    #[test]
    fn every_part_of_a_table_reports_to_one_bar() {
        let tracker = Arc::new(RecordingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let sessions = (1..=4)
            .map(|part| (Table::StoreSales, chunk(PARTITIONABLE_SCALE, part, 4)))
            .collect();

        let work = plan_tables(sessions, CHUNK_BYTES, ChunkFormat::Dat, &progress);

        let registered = tracker.registered.lock().unwrap();
        assert_eq!(registered.len(), 1, "expected a single store_sales bar");
        assert_eq!(registered[0].0, "store_sales");
        // The bar's total is the sum of what its parts will generate
        let planned_chunks: usize = work.iter().map(|w| w.plan.chunk_count()).sum();
        assert_eq!(registered[0].1, planned_chunks as u64);
    }

    #[test]
    fn all_parts_together_plan_the_whole_table() {
        let tracker = Arc::new(RecordingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker.clone();

        let one = plan_tables(
            vec![(Table::StoreSales, whole(PARTITIONABLE_SCALE))],
            CHUNK_BYTES,
            ChunkFormat::Dat,
            &progress,
        );
        let four = plan_tables(
            (1..=4)
                .map(|part| (Table::StoreSales, chunk(PARTITIONABLE_SCALE, part, 4)))
                .collect(),
            CHUNK_BYTES,
            ChunkFormat::Dat,
            &progress,
        );

        assert_eq!(one.len(), 1);
        assert_eq!(four.len(), 4);
        let one_chunks: usize = one.iter().map(|w| w.plan.chunk_count()).sum();
        let four_chunks: usize = four.iter().map(|w| w.plan.chunk_count()).sum();
        // splitting into parts rounds each part up, so allow a chunk per part
        assert!(
            four_chunks >= one_chunks && four_chunks <= one_chunks + 4,
            "{four_chunks} chunks across 4 parts vs {one_chunks} for the whole table"
        );
    }

    #[test]
    fn empty_parts_are_dropped() {
        // `reason` is far below dsdgen's 1M source row threshold, so chunk 1
        // generates all of it and the other chunks have no file to write.
        let tracker = Arc::new(RecordingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let sessions = (1..=4)
            .map(|part| (Table::Reason, chunk(1.0, part, 4)))
            .collect();

        let work = plan_tables(sessions, CHUNK_BYTES, ChunkFormat::Csv, &progress);

        assert_eq!(work.len(), 1);
        assert_eq!(work[0].session.get_chunk_number(), 1);
    }

    #[tokio::test]
    async fn run_plans_generates_every_planned_table() {
        let tracker = Arc::new(RecordingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let work = plan_tables(
            vec![
                (Table::Reason, whole(1.0)),
                (Table::ShipMode, whole(1.0)),
                (Table::Store, whole(1.0)),
            ],
            CHUNK_BYTES,
            ChunkFormat::Dat,
            &progress,
        );
        assert_eq!(work.len(), 3);

        let generated = Arc::new(AtomicU64::new(0));
        let counter = Arc::clone(&generated);
        run_plans(work, 2, move |_planned, _num_threads| {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        })
        .await
        .unwrap();

        assert_eq!(generated.load(Ordering::Relaxed), 3);
    }
}
