//! Drivers for the TPC-DS row generators, shared by the DAT and CSV outputs.

use super::runner::PlannedTable;
use crate::generate::{Source, TextOutput};
use crate::output_location::OutputLocation;
use log::info;
use std::io;
use std::ops::RangeInclusive;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::*;

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

/// Trait for formatting text output for the TPC-DS row generators (DAT or CSV).
///
/// Generic over the row type `R`: each table's generator produces its own
/// concrete row type (e.g. `InventoryRow`).
pub(super) trait RowFormat<R>: Clone + Send + 'static {
    /// The file extension of this format's output files.
    const EXTENSION: &'static str;

    /// Write the header line for `table`, if this format has one. Called once
    /// per file, before any rows.
    fn write_header(&self, table: Table, buffer: Vec<u8>) -> Vec<u8>;

    /// Format `rows` (all belonging to `table`) into `buffer`.
    fn write_rows<I>(&self, table: Table, rows: I, buffer: Vec<u8>) -> Vec<u8>
    where
        I: Iterator<Item = R>;
}

/// A [`RowFormat`] for every TPC-DS row type.
macro_rules! all_row_formats {
    ($($row:ty),* $(,)?) => {
        pub(super) trait AllRowFormats: $(RowFormat<$row> +)* Clone {}
        impl<F: $(RowFormat<$row> +)* Clone> AllRowFormats for F {}
    };
}

all_row_formats!(
    CallCenterRow,
    CatalogPageRow,
    CatalogReturnsRow,
    CatalogSalesRow,
    CustomerRow,
    CustomerAddressRow,
    CustomerDemographicsRow,
    DateDimRow,
    DbgenVersionRow,
    HouseholdDemographicsRow,
    IncomeBandRow,
    InventoryRow,
    ItemRow,
    PromotionRow,
    ReasonRow,
    ShipModeRow,
    StoreRow,
    StoreReturnsRow,
    StoreSalesRow,
    TimeDimRow,
    WarehouseRow,
    WebPageRow,
    WebReturnsRow,
    WebSalesRow,
    WebSiteRow,
);

/// Generate one planned table (one `--parts` chunk of one table) into
/// `base_location`, using up to `num_threads` threads.
///
/// A sales generator emits rows for its returns table too; each output keeps
/// only its own rows, the same way the Arrow generators produce them.
pub(super) async fn generate_table<F: AllRowFormats>(
    format: F,
    base_location: OutputLocation,
    planned: PlannedTable,
    num_threads: usize,
) -> io::Result<()> {
    // One concrete row per source row
    macro_rules! single {
        ($GENERATOR:ty) => {
            write_table(
                format,
                &base_location,
                planned,
                num_threads,
                |session, source_rows, range| {
                    let mut rows = SingleRowIter::new(<$GENERATOR>::new(), session, source_rows);
                    rows.set_source_row_range(*range.start(), *range.end());
                    rows
                },
            )
            .await
        };
    }
    // Sales generators emit a sales row and maybe a returns row per line item
    macro_rules! sales {
        ($generator:expr, $select:expr) => {
            write_table(
                format,
                &base_location,
                planned,
                num_threads,
                |session, source_rows, range| {
                    let mut rows = SalesRowIter::new($generator, session, source_rows);
                    rows.set_source_row_range(*range.start(), *range.end());
                    rows.filter_map($select)
                },
            )
            .await
        };
    }

    match planned.table {
        // Simple dimension tables
        Table::CallCenter => single!(CallCenterRowGenerator),
        Table::CatalogPage => single!(CatalogPageRowGenerator),
        Table::Customer => single!(CustomerRowGenerator),
        Table::CustomerAddress => single!(CustomerAddressRowGenerator),
        Table::CustomerDemographics => single!(CustomerDemographicsRowGenerator),
        Table::DateDim => single!(DateDimRowGenerator),
        Table::DbgenVersion => single!(DbgenVersionRowGenerator),
        Table::HouseholdDemographics => single!(HouseholdDemographicsRowGenerator),
        Table::IncomeBand => single!(IncomeBandRowGenerator),
        Table::Inventory => single!(InventoryRowGenerator),
        Table::Item => single!(ItemRowGenerator),
        Table::Promotion => single!(PromotionRowGenerator),
        Table::Reason => single!(ReasonRowGenerator),
        Table::ShipMode => single!(ShipModeRowGenerator),
        Table::Store => single!(StoreRowGenerator),
        Table::TimeDim => single!(TimeDimRowGenerator),
        Table::Warehouse => single!(WarehouseRowGenerator),
        Table::WebPage => single!(WebPageRowGenerator),
        Table::WebSite => single!(WebSiteRowGenerator),

        // Sales tables and the returns tables their generator also emits
        Table::StoreSales => sales!(StoreSalesRowGenerator::sales(), |r| r.sales),
        Table::StoreReturns => sales!(StoreSalesRowGenerator::returns(), |r| r.returns),
        Table::CatalogSales => sales!(CatalogSalesRowGenerator::new(), |r| r.sales),
        Table::CatalogReturns => sales!(CatalogSalesRowGenerator::new(), |r| r.returns),
        Table::WebSales => sales!(WebSalesRowGenerator::new(), |r| r.sales),
        Table::WebReturns => sales!(WebSalesRowGenerator::new(), |r| r.returns),

        // Source tables - skip
        _ => Ok(()),
    }
}

/// Generate the rows in `planned`, where `make_rows` creates the row
/// iterator for one chunk: `(session, source_rows, range)`.
///
/// Progress is counted in chunks; the totals are registered by
/// [`super::runner::plan_tables`]
async fn write_table<F, I>(
    format: F,
    base_location: &OutputLocation,
    planned: PlannedTable,
    num_threads: usize,
    make_rows: fn(Session, u64, RangeInclusive<u64>) -> I,
) -> io::Result<()>
where
    F: RowFormat<I::Item>,
    I: Iterator + 'static,
{
    let PlannedTable {
        table,
        session,
        plan,
        progress,
    } = planned;

    let location = output_location_for_table(base_location, table, F::EXTENSION, &session)?;
    let chunk_count = plan.chunk_count() as u64;
    let scale_factor = session.get_scaling().get_scale();
    let part = session.get_chunk_number();
    let parts = session.get_total_chunks();
    let partition = if session.is_partitioned() {
        format!(" (part {part}/{parts})")
    } else {
        String::new()
    };
    let source_rows = session.get_scaling().get_row_count(table.source_table());
    let sources = plan.into_iter().map(move |range| RowSource {
        format: format.clone(),
        table,
        session: session.clone(),
        source_rows,
        range,
        make_rows,
    });

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

/// Generates the text for one chunk (a range of source rows) of one table.
struct RowSource<F, I> {
    format: F,
    table: Table,
    session: Session,
    source_rows: u64,
    /// The 1-based inclusive source rows of this chunk
    range: RangeInclusive<u64>,
    make_rows: fn(Session, u64, RangeInclusive<u64>) -> I,
}

impl<F, I> Source for RowSource<F, I>
where
    F: RowFormat<I::Item>,
    I: Iterator + 'static,
{
    fn header(&self, buffer: Vec<u8>) -> Vec<u8> {
        self.format.write_header(self.table, buffer)
    }

    fn create(self, buffer: Vec<u8>) -> Vec<u8> {
        let Self {
            format,
            table,
            session,
            source_rows,
            range,
            make_rows,
        } = self;

        let rows = make_rows(session, source_rows, range);
        format.write_rows(table, rows, buffer)
    }
}
