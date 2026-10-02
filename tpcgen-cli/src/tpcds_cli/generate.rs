//! Drivers for the TPC-DS outputs (DAT, CSV and Parquet).
//!
//! As in [`crate::tpch_cli::runner`], each table has one function, defined by
//! `define_run!`, that names the table's row generator, CSV wrapper and Arrow
//! reader and dispatches on the requested output format.

use super::csv::*;
use super::dat::*;
use super::plan::ChunkFormat;
use super::runner::{plan_tables, run_plans, PlannedTable};
use super::OutputFormat;
use crate::generate::{Source, TextOutput};
use crate::output_location::OutputLocation;
use crate::progress::ProgressTracker;
use log::info;
use std::io;
use std::ops::RangeInclusive;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::*;
use tpcdsgen_arrow::*;

impl OutputFormat {
    fn chunk_format(&self) -> ChunkFormat {
        match self {
            Self::Dat(_) => ChunkFormat::Dat,
            Self::Csv(_) => ChunkFormat::Csv,
            Self::Parquet(_) => ChunkFormat::Parquet,
        }
    }

    fn chunk_size_bytes(&self) -> i64 {
        match self {
            Self::Dat(dat) => dat.chunk_size_bytes,
            Self::Csv(csv) => csv.chunk_size_bytes,
            Self::Parquet(parquet) => parquet.row_group_bytes,
        }
    }

    /// Generate the given TPC-DS tables, one file per `(table, session)`.
    pub(super) async fn generate_tables(
        self,
        table_sessions: Vec<(Table, Session)>,
        num_threads: usize,
        progress: Arc<dyn ProgressTracker>,
    ) -> io::Result<()> {
        if let Self::Parquet(parquet) = &self {
            parquet.validate(&table_sessions)?;
        }

        let work = plan_tables(
            table_sessions,
            self.chunk_size_bytes(),
            self.chunk_format(),
            &progress,
        );
        progress.start();

        let format = Arc::new(self);
        run_plans(work, num_threads, move |planned, num_threads| {
            let format = Arc::clone(&format);
            async move { run_plan(&format, planned, num_threads).await }
        })
        .await
    }
}

/// Return the output location for `table`, relative to `base_location`.
///
/// When `--parts` was not requested creates a single `<table>.<ext>` file, otherwise
/// written into a subdirectory like `<table>/<table>.<chunk>.<ext>`.
///
/// Note that `--parts 1` is also written to a subdirectory.
///
/// This function creates the per-table subdirectory as needed. Writing to
/// stdout creates no directories: every table shares the one stream.
pub(super) fn output_location_for_table(
    base_location: &OutputLocation,
    table: Table,
    ext: &str,
    session: &Session,
) -> io::Result<OutputLocation> {
    // sub directory `<table>/<table>.<chunk>.<ext>`
    if session.is_partitioned() {
        let dir = base_location.join(table.get_name());
        dir.create_dir_all()?;
        Ok(dir.join(format!(
            "{}.{}.{ext}",
            table.get_name(),
            session.get_chunk_number()
        )))
    } else {
        // single `<table>.<ext>` file
        Ok(base_location.join(format!("{}.{ext}", table.get_name())))
    }
}

/// Generate one planned table (one `--parts` chunk of one table) in `format`,
/// using up to `num_threads` threads.
async fn run_plan(
    format: &OutputFormat,
    planned: PlannedTable,
    num_threads: usize,
) -> io::Result<()> {
    match planned.table {
        Table::CallCenter => run_call_center(format, planned, num_threads).await,
        Table::CatalogPage => run_catalog_page(format, planned, num_threads).await,
        Table::CatalogReturns => run_catalog_returns(format, planned, num_threads).await,
        Table::CatalogSales => run_catalog_sales(format, planned, num_threads).await,
        Table::Customer => run_customer(format, planned, num_threads).await,
        Table::CustomerAddress => run_customer_address(format, planned, num_threads).await,
        Table::CustomerDemographics => {
            run_customer_demographics(format, planned, num_threads).await
        }
        Table::DateDim => run_date_dim(format, planned, num_threads).await,
        Table::DbgenVersion => run_dbgen_version(format, planned, num_threads).await,
        Table::HouseholdDemographics => {
            run_household_demographics(format, planned, num_threads).await
        }
        Table::IncomeBand => run_income_band(format, planned, num_threads).await,
        Table::Inventory => run_inventory(format, planned, num_threads).await,
        Table::Item => run_item(format, planned, num_threads).await,
        Table::Promotion => run_promotion(format, planned, num_threads).await,
        Table::Reason => run_reason(format, planned, num_threads).await,
        Table::ShipMode => run_ship_mode(format, planned, num_threads).await,
        Table::Store => run_store(format, planned, num_threads).await,
        Table::StoreReturns => run_store_returns(format, planned, num_threads).await,
        Table::StoreSales => run_store_sales(format, planned, num_threads).await,
        Table::TimeDim => run_time_dim(format, planned, num_threads).await,
        Table::Warehouse => run_warehouse(format, planned, num_threads).await,
        Table::WebPage => run_web_page(format, planned, num_threads).await,
        Table::WebReturns => run_web_returns(format, planned, num_threads).await,
        Table::WebSales => run_web_sales(format, planned, num_threads).await,
        Table::WebSite => run_web_site(format, planned, num_threads).await,
        // Source tables - skip
        _ => Ok(()),
    }
}

/// Define the function that generates one planned table in whichever format
/// was requested.
///
/// Arguments:
/// `$FUN_NAME`: name of the function to create
/// `$GENERATOR`: the table's row generator type. A sales or returns generator
///   iterates over its source rows directly, named as `$GENERATOR: Iterator`
/// `$DAT_SOURCE`: the [`Source`] type to use for DAT format
/// `$CSV_SOURCE`: the [`Source`] type to use for CSV format
/// `$PARQUET_SOURCE`: the [`arrow::record_batch::RecordBatchReader`] type to use for Parquet format
macro_rules! define_run {
    ($FUN_NAME:ident, $GENERATOR:ident, $DAT_SOURCE:ty, $CSV_SOURCE:ty, $PARQUET_SOURCE:ty) => {
        define_run!(@rows $FUN_NAME, SingleRowIter<$GENERATOR>,
            |session, source_rows| SingleRowIter::new(<$GENERATOR>::new(), session, source_rows),
            $DAT_SOURCE, $CSV_SOURCE, $PARQUET_SOURCE);
    };
    ($FUN_NAME:ident, $GENERATOR:ident: Iterator, $DAT_SOURCE:ty, $CSV_SOURCE:ty, $PARQUET_SOURCE:ty) => {
        define_run!(@rows $FUN_NAME, $GENERATOR, <$GENERATOR>::new,
            $DAT_SOURCE, $CSV_SOURCE, $PARQUET_SOURCE);
    };
    // `$ROWS` iterates the rows of one chunk, built by `$new_rows(session, source_rows)`
    (@rows $FUN_NAME:ident, $ROWS:ty, $new_rows:expr, $DAT_SOURCE:ty, $CSV_SOURCE:ty, $PARQUET_SOURCE:ty) => {
        async fn $FUN_NAME(
            format: &OutputFormat,
            planned: PlannedTable,
            num_threads: usize,
        ) -> io::Result<()> {
            /// The rows of one chunk: source rows `range` of `source_rows`
            fn rows(session: Session, source_rows: u64, range: RangeInclusive<u64>) -> $ROWS {
                let mut rows = $new_rows(session, source_rows);
                rows.set_source_row_range(*range.start(), *range.end());
                rows
            }

            match format {
                OutputFormat::Dat(dat) => {
                    let compat_mode = dat.compat_mode;
                    let sources = planned.chunks().map(move |(session, source_rows, range)| {
                        <$DAT_SOURCE>::new(rows(session, source_rows, range), compat_mode)
                    });
                    write_text(&dat.base_location, "dat", planned, num_threads, sources).await
                }
                OutputFormat::Csv(csv) => {
                    let delimiter = csv.delimiter;
                    let sources = planned.chunks().map(move |(session, source_rows, range)| {
                        <$CSV_SOURCE>::new(rows(session, source_rows, range), delimiter)
                    });
                    write_text(&csv.base_location, "csv", planned, num_threads, sources).await
                }
                OutputFormat::Parquet(parquet) => {
                    parquet
                        .write_table(planned, num_threads, |session, start, end| {
                            <$PARQUET_SOURCE>::new(session).with_source_row_range(start, end)
                        })
                        .await
                }
            }
        }
    };
}

define_run!(
    run_call_center,
    CallCenterRowGenerator,
    CallCenterDatSource,
    CallCenterCsvSource,
    CallCenterArrow
);
define_run!(
    run_catalog_page,
    CatalogPageRowGenerator,
    CatalogPageDatSource,
    CatalogPageCsvSource,
    CatalogPageArrow
);
define_run!(
    run_catalog_returns,
    CatalogReturnsRowGenerator: Iterator,
    CatalogReturnsDatSource,
    CatalogReturnsCsvSource,
    CatalogReturnsArrow
);
define_run!(
    run_catalog_sales,
    CatalogSalesRowGenerator: Iterator,
    CatalogSalesDatSource,
    CatalogSalesCsvSource,
    CatalogSalesArrow
);
define_run!(
    run_customer,
    CustomerRowGenerator,
    CustomerDatSource,
    CustomerCsvSource,
    CustomerArrow
);
define_run!(
    run_customer_address,
    CustomerAddressRowGenerator,
    CustomerAddressDatSource,
    CustomerAddressCsvSource,
    CustomerAddressArrow
);
define_run!(
    run_customer_demographics,
    CustomerDemographicsRowGenerator,
    CustomerDemographicsDatSource,
    CustomerDemographicsCsvSource,
    CustomerDemographicsArrow
);
define_run!(
    run_date_dim,
    DateDimRowGenerator,
    DateDimDatSource,
    DateDimCsvSource,
    DateDimArrow
);
define_run!(
    run_dbgen_version,
    DbgenVersionRowGenerator,
    DbgenVersionDatSource,
    DbgenVersionCsvSource,
    DbgenVersionArrow
);
define_run!(
    run_household_demographics,
    HouseholdDemographicsRowGenerator,
    HouseholdDemographicsDatSource,
    HouseholdDemographicsCsvSource,
    HouseholdDemographicsArrow
);
define_run!(
    run_income_band,
    IncomeBandRowGenerator,
    IncomeBandDatSource,
    IncomeBandCsvSource,
    IncomeBandArrow
);
define_run!(
    run_inventory,
    InventoryRowGenerator,
    InventoryDatSource,
    InventoryCsvSource,
    InventoryArrow
);
define_run!(
    run_item,
    ItemRowGenerator,
    ItemDatSource,
    ItemCsvSource,
    ItemArrow
);
define_run!(
    run_promotion,
    PromotionRowGenerator,
    PromotionDatSource,
    PromotionCsvSource,
    PromotionArrow
);
define_run!(
    run_reason,
    ReasonRowGenerator,
    ReasonDatSource,
    ReasonCsvSource,
    ReasonArrow
);
define_run!(
    run_ship_mode,
    ShipModeRowGenerator,
    ShipModeDatSource,
    ShipModeCsvSource,
    ShipModeArrow
);
define_run!(
    run_store,
    StoreRowGenerator,
    StoreDatSource,
    StoreCsvSource,
    StoreArrow
);
define_run!(
    run_store_returns,
    StoreReturnsRowGenerator: Iterator,
    StoreReturnsDatSource,
    StoreReturnsCsvSource,
    StoreReturnsArrow
);
define_run!(
    run_store_sales,
    StoreSalesRowGenerator: Iterator,
    StoreSalesDatSource,
    StoreSalesCsvSource,
    StoreSalesArrow
);
define_run!(
    run_time_dim,
    TimeDimRowGenerator,
    TimeDimDatSource,
    TimeDimCsvSource,
    TimeDimArrow
);
define_run!(
    run_warehouse,
    WarehouseRowGenerator,
    WarehouseDatSource,
    WarehouseCsvSource,
    WarehouseArrow
);
define_run!(
    run_web_page,
    WebPageRowGenerator,
    WebPageDatSource,
    WebPageCsvSource,
    WebPageArrow
);
define_run!(
    run_web_returns,
    WebReturnsRowGenerator: Iterator,
    WebReturnsDatSource,
    WebReturnsCsvSource,
    WebReturnsArrow
);
define_run!(
    run_web_sales,
    WebSalesRowGenerator: Iterator,
    WebSalesDatSource,
    WebSalesCsvSource,
    WebSalesArrow
);
define_run!(
    run_web_site,
    WebSiteRowGenerator,
    WebSiteDatSource,
    WebSiteCsvSource,
    WebSiteArrow
);

/// Write `sources`, the chunks of `planned`, as one text file.
///
/// Progress is counted in chunks; the totals are registered by
/// [`super::runner::plan_tables`]
async fn write_text<I>(
    base_location: &OutputLocation,
    extension: &str,
    planned: PlannedTable,
    num_threads: usize,
    sources: I,
) -> io::Result<()>
where
    I: Iterator<Item: Source + 'static> + Send + 'static,
{
    let PlannedTable {
        table,
        session,
        plan,
        progress,
    } = planned;

    let location = output_location_for_table(base_location, table, extension, &session)?;
    let chunk_count = plan.chunk_count() as u64;
    let scale_factor = session.get_scaling().get_scale();
    let part = session.get_chunk_number();
    let parts = session.get_total_chunks();
    let partition = if session.is_partitioned() {
        format!(" (part {part}/{parts})")
    } else {
        String::new()
    };

    info!(
        "Writing table {table} (SF={scale_factor}, {chunk_count} chunk{}){partition} to {location} using {num_threads} thread{}",
        if chunk_count == 1 { "" } else { "s" },
        if num_threads == 1 { "" } else { "s" }
    );
    let written = location
        .write(TextOutput {
            sources,
            num_threads,
            progress: progress.clone(),
        })
        .await?;
    if written {
        info!("Generated table {table}{partition} to {location}");
    } else {
        // Skipped, so count all chunks at once
        progress.increment(chunk_count, 0);
    }
    progress.complete();
    Ok(())
}
