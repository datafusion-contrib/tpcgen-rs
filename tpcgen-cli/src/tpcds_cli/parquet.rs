//! TPC-DS Parquet output.

use super::plan::TpcdsGenerationPlan;
use crate::parquet::generate_parquet;
use crate::progress::{ProgressHandle, ProgressTracker};
use crate::temp_path::inprogress_path;
use crate::worker_queue::WorkerQueue;
use arrow::record_batch::RecordBatchReader;
use parquet::basic::Compression;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

/// Parquet output generator.
#[derive(Debug, Clone)]
pub(super) struct Parquet {
    output_dir: PathBuf,
    compression: Compression,
    row_group_bytes: usize,
    num_threads: usize,
}

impl Parquet {
    pub(super) fn new(
        output_dir: PathBuf,
        compression: Compression,
        row_group_bytes: usize,
        num_threads: usize,
    ) -> Self {
        Self {
            output_dir,
            compression,
            row_group_bytes,
            num_threads,
        }
    }

    /// Generate the given TPC-DS tables as Parquet files.
    ///
    /// Tables are generated concurrently: each table's plan gets as many
    /// threads as it has row groups, within the overall `num_threads`
    /// budget (see [`WorkerQueue`]). Scheduling the largest tables first
    /// keeps all cores busy while the trailing row groups of each table
    /// are encoded, instead of waiting for one table at a time.
    pub(super) async fn generate_tables(
        &self,
        tables: Vec<(Table, Session)>,
        progress: Arc<dyn ProgressTracker>,
    ) -> io::Result<()> {
        // Plan each table and pre-register the row group totals so trackers
        // can size their bars before the first increment
        let mut work: Vec<(Table, Session, TpcdsGenerationPlan, ProgressHandle)> = tables
            .into_iter()
            .map(|(table, session)| {
                let plan =
                    TpcdsGenerationPlan::new(table, session.get_scaling(), self.row_group_bytes);
                let progress = progress
                    .clone()
                    .register(table.get_name(), plan.row_group_count() as u64);
                (table, session, plan, progress)
            })
            .collect();
        progress.start();

        // Schedule the largest tables (most row groups) first for the best
        // thread utilization (the list is popped from the back)
        work.sort_by_key(|(_, _, plan, _)| plan.row_group_count());

        let mut queue = WorkerQueue::new(self.num_threads);
        while let Some((table, session, plan, progress)) = work.pop() {
            let this = self.clone();
            queue
                .schedule(plan.row_group_count(), move |num_threads| async move {
                    this.generate_table(table, session, plan, num_threads, progress)
                        .await?;
                    Ok(num_threads)
                })
                .await?;
        }
        queue.join_all().await
    }

    /// Generate one TPC-DS table as a Parquet file using `num_threads`
    /// threads.
    async fn generate_table(
        &self,
        table: Table,
        session: Session,
        plan: TpcdsGenerationPlan,
        num_threads: usize,
        progress: ProgressHandle,
    ) -> io::Result<()> {
        match table {
            Table::CallCenter => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CallCenterArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::CatalogPage => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CatalogPageArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::CatalogReturns => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CatalogReturnsArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::CatalogSales => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CatalogSalesArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::Customer => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CustomerArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::CustomerAddress => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CustomerAddressArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::CustomerDemographics => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        CustomerDemographicsArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::DateDim => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        DateDimArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::DbgenVersion => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        DbgenVersionArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::HouseholdDemographics => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        HouseholdDemographicsArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::IncomeBand => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        IncomeBandArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::Inventory => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        InventoryArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::Item => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| ItemArrow::new(session).with_source_row_range(start, end),
                )
                .await
            }
            Table::Promotion => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        PromotionArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::Reason => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        ReasonArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::ShipMode => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        ShipModeArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::Store => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        StoreArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::StoreReturns => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        StoreReturnsArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::StoreSales => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        StoreSalesArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::TimeDim => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        TimeDimArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::Warehouse => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        WarehouseArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::WebPage => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        WebPageArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::WebReturns => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        WebReturnsArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::WebSales => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        WebSalesArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            Table::WebSite => {
                self.write_table(
                    table,
                    session,
                    plan,
                    num_threads,
                    progress,
                    |session, start, end| {
                        WebSiteArrow::new(session).with_source_row_range(start, end)
                    },
                )
                .await
            }
            _ => Ok(()),
        }
    }

    /// Write one table to a Parquet file at the specified path.
    ///
    /// `make_reader` creates a [`RecordBatchReader`] for one planned source
    /// row range; the batches of each reader are encoded (in parallel, using
    /// up to `num_threads` threads) as one row group.
    ///
    /// Progress is reported in row groups: the shared writer advances by
    /// one per written row group (the same output units as TPC-H parquet
    /// generation; the totals are registered in [`Self::generate_tables`]).
    async fn write_table<R, F>(
        &self,
        table: Table,
        session: Session,
        plan: TpcdsGenerationPlan,
        num_threads: usize,
        progress: ProgressHandle,
        make_reader: F,
    ) -> io::Result<()>
    where
        R: RecordBatchReader + Send + 'static,
        F: Fn(Session, u64, u64) -> R + Send + 'static,
    {
        let table_name = table.get_name();
        let path = self.output_dir.join(format!("{table_name}.parquet"));
        let sources = plan
            .into_iter()
            .map(move |range| make_reader(session.clone(), *range.start(), *range.end()));

        // write to a temp file and then rename to avoid partial files
        let temp_path = inprogress_path(&path);
        let file = File::create(&temp_path)
            .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
        let writer = BufWriter::with_capacity(32 * 1024 * 1024, file);
        generate_parquet(
            writer,
            sources,
            num_threads,
            self.compression,
            progress.clone(),
        )
        .await?;
        std::fs::rename(&temp_path, &path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {temp_path:?} to {path:?} file: {err}"
            ))
        })?;
        progress.complete();

        Ok(())
    }
}
