//! TPC-DS Parquet output.

use super::plan::TpcdsGenerationPlan;
use crate::parquet::generate_parquet;
use crate::progress::{ProgressHandle, ProgressTracker};
use crate::temp_path::inprogress_path;
use crate::worker_queue::WorkerQueue;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatchReader;
use parquet::basic::{Compression, Encoding};
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

/// Returns `table`'s Arrow schema. Does not generate any rows.
fn table_schema(table: Table, session: &Session) -> SchemaRef {
    let session = session.clone();
    match table {
        Table::CallCenter => CallCenterArrow::new(session).schema(),
        Table::CatalogPage => CatalogPageArrow::new(session).schema(),
        Table::CatalogReturns => CatalogReturnsArrow::new(session).schema(),
        Table::CatalogSales => CatalogSalesArrow::new(session).schema(),
        Table::Customer => CustomerArrow::new(session).schema(),
        Table::CustomerAddress => CustomerAddressArrow::new(session).schema(),
        Table::CustomerDemographics => CustomerDemographicsArrow::new(session).schema(),
        Table::DateDim => DateDimArrow::new(session).schema(),
        Table::DbgenVersion => DbgenVersionArrow::new(session).schema(),
        Table::HouseholdDemographics => HouseholdDemographicsArrow::new(session).schema(),
        Table::IncomeBand => IncomeBandArrow::new(session).schema(),
        Table::Inventory => InventoryArrow::new(session).schema(),
        Table::Item => ItemArrow::new(session).schema(),
        Table::Promotion => PromotionArrow::new(session).schema(),
        Table::Reason => ReasonArrow::new(session).schema(),
        Table::ShipMode => ShipModeArrow::new(session).schema(),
        Table::Store => StoreArrow::new(session).schema(),
        Table::StoreReturns => StoreReturnsArrow::new(session).schema(),
        Table::StoreSales => StoreSalesArrow::new(session).schema(),
        Table::TimeDim => TimeDimArrow::new(session).schema(),
        Table::Warehouse => WarehouseArrow::new(session).schema(),
        Table::WebPage => WebPageArrow::new(session).schema(),
        Table::WebReturns => WebReturnsArrow::new(session).schema(),
        Table::WebSales => WebSalesArrow::new(session).schema(),
        Table::WebSite => WebSiteArrow::new(session).schema(),
        _ => unreachable!("table_schema is only called for main TPC-DS tables"),
    }
}

/// Checks each column in `encodings` against every table in `tables`.
///
/// Rejects an encoding `reject_unsupported_encoding` always rejects.
/// Rejects a column name that matches no table (almost always a typo). A
/// column that matches only some tables is fine: [`column_encodings_for_table`]
/// applies it there and skips it elsewhere.
fn validate_column_encodings(
    tables: &[(Table, Session)],
    encodings: &[(String, Encoding)],
) -> io::Result<()> {
    for (col, enc) in encodings {
        crate::parquet::reject_unsupported_encoding(*enc)?;
        let matches_any_table = tables.iter().any(|(table, session)| {
            table_schema(*table, session)
                .fields()
                .iter()
                .any(|f| f.name() == col)
        });
        if !matches_any_table {
            return Err(io::Error::other(format!(
                "column '{col}' for --column-encoding not found in any selected table"
            )));
        }
    }
    Ok(())
}

/// Keeps only the encodings whose column exists in `table`'s schema.
fn column_encodings_for_table(
    table: Table,
    session: &Session,
    encodings: &[(String, Encoding)],
) -> Vec<(String, Encoding)> {
    let schema = table_schema(table, session);
    encodings
        .iter()
        .filter(|(col, _)| schema.fields().iter().any(|f| f.name() == col))
        .cloned()
        .collect()
}

/// Parquet output generator.
#[derive(Debug, Clone)]
pub(super) struct Parquet {
    output_dir: PathBuf,
    compression: Compression,
    row_group_bytes: usize,
    num_threads: usize,
    column_encodings: Option<Vec<(String, Encoding)>>,
}

impl Parquet {
    pub(super) fn new(
        output_dir: PathBuf,
        compression: Compression,
        row_group_bytes: usize,
        num_threads: usize,
        column_encodings: Option<Vec<(String, Encoding)>>,
    ) -> Self {
        Self {
            output_dir,
            compression,
            row_group_bytes,
            num_threads,
            column_encodings,
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
        // Reject a --column-encoding column that matches no selected table
        // (a typo) before any work starts. column_encodings_for_table
        // (below) skips a column that only matches some tables, so that
        // case is not an error.
        if let Some(encodings) = &self.column_encodings {
            validate_column_encodings(&tables, encodings)?;
        }

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
        F: Fn(Session, i64, i64) -> R + Send + 'static,
    {
        let table_name = table.get_name();
        let path = self.output_dir.join(format!("{table_name}.parquet"));

        // Keep only the encodings for columns on this table.
        // --column-encoding usually targets a few tables, not all of them.
        let column_encodings = self
            .column_encodings
            .as_ref()
            .map(|encodings| column_encodings_for_table(table, &session, encodings));

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
            column_encodings.as_deref(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn table_sessions(tables: &[Table]) -> Vec<(Table, Session)> {
        tables
            .iter()
            .map(|&table| (table, Session::default()))
            .collect()
    }

    #[test]
    fn validate_column_encodings_accepts_a_column_present_on_just_one_table() {
        // r_reason_description exists only on reason, not item.
        let tables = table_sessions(&[Table::Reason, Table::Item]);
        let encodings = [("r_reason_description".to_string(), Encoding::PLAIN)];
        assert!(validate_column_encodings(&tables, &encodings).is_ok());
    }

    #[test]
    fn validate_column_encodings_rejects_a_typo() {
        let tables = table_sessions(&[Table::Reason]);
        let encodings = [("r_reason_description_typo".to_string(), Encoding::PLAIN)];
        let err = validate_column_encodings(&tables, &encodings).unwrap_err();
        assert!(
            err.to_string()
                .contains("column 'r_reason_description_typo'"),
            "{err}"
        );
    }

    #[test]
    fn validate_column_encodings_rejects_dictionary_encoding() {
        let tables = table_sessions(&[Table::Reason]);
        let encodings = [(
            "r_reason_description".to_string(),
            Encoding::PLAIN_DICTIONARY,
        )];
        assert!(validate_column_encodings(&tables, &encodings).is_err());
    }

    #[test]
    fn column_encodings_for_table_keeps_only_matching_columns() {
        let session = Session::default();
        let encodings = [
            ("r_reason_description".to_string(), Encoding::PLAIN),
            ("i_item_desc".to_string(), Encoding::PLAIN),
        ];
        assert_eq!(
            column_encodings_for_table(Table::Reason, &session, &encodings),
            vec![("r_reason_description".to_string(), Encoding::PLAIN)]
        );
        assert_eq!(
            column_encodings_for_table(Table::CallCenter, &session, &encodings),
            Vec::new()
        );
    }
}
