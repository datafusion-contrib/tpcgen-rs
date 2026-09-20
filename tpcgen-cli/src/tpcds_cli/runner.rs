//! Planning and scheduling TPC-DS table generation.
//!
//! This mirrors [`crate::tpch_cli::runner`], which does the same job for the
//! TPC-H outputs.

use super::plan::TpcdsGenerationPlan;
use super::progress::share_handle_across_parts;
use crate::progress::{ProgressHandle, ProgressTracker};
use crate::worker_queue::WorkerQueue;
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};

/// One unit of schedulable work: the chunks of one table, together with the
/// progress handle they report to.
#[derive(Debug)]
pub(super) struct PlannedTable {
    /// The table to write
    pub(super) table: Table,
    /// The session for this chunk
    pub(super) session: Session,
    /// How the source rows are split into chunks
    pub(super) plan: TpcdsGenerationPlan,
    /// Progress reporter
    pub(super) progress: ProgressHandle,
}

/// Plan every requested `(table, session)` and register the progress bars.
pub(super) fn plan_tables(
    table_sessions: Vec<(Table, Session)>,
    chunk_size_bytes: i64,
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
                let plan = TpcdsGenerationPlan::new_for_range(table, chunk_size_bytes, row_range);
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

/// Generate every [`PlannedTable`] using `num_threads` threads.
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
