//! Drivers for the TPC-DS row generators, shared by the DAT and CSV outputs.

use super::runner::PlannedTable;
use crate::generate::{generate_file, generate_in_chunks, Source};
use crate::output_location::OutputLocation;
use crate::sink::WriterSink;
use log::info;
use std::io;
use std::marker::PhantomData;
use std::ops::RangeInclusive;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::*;

/// Return the output location for `table`, relative to `base_location` (the
/// output directory, or stdout), following `tpchgen-cli`'s `--parts`/`--part`
/// naming convention:
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
pub(super) trait RowFormat: Clone + Send + 'static {
    /// The file extension of this format's output files.
    const EXTENSION: &'static str;

    /// Write the header line for `table`, if this format has one. Called once
    /// per file, before any rows.
    fn write_header(&self, table: Table, buffer: Vec<u8>) -> Vec<u8>;

    /// Format `rows` (all belonging to `table`) into `buffer`.
    fn write_rows<I>(&self, table: Table, rows: I, buffer: Vec<u8>) -> Vec<u8>
    where
        I: Iterator<Item = GeneratedRow>;
}

/// Trait for creating row generators.
pub(super) trait RowGeneratorFactory: RowGenerator + Sized {
    fn create() -> Self;
}

macro_rules! impl_factory {
    ($($gen:ty),*) => {
        $(
            impl RowGeneratorFactory for $gen {
                fn create() -> Self { Self::new() }
            }
        )*
    };
}

// Implement factory for all simple generators
impl_factory!(
    CallCenterRowGenerator,
    CatalogPageRowGenerator,
    CustomerRowGenerator,
    CustomerAddressRowGenerator,
    CustomerDemographicsRowGenerator,
    DateDimRowGenerator,
    DbgenVersionRowGenerator,
    HouseholdDemographicsRowGenerator,
    IncomeBandRowGenerator,
    InventoryRowGenerator,
    ItemRowGenerator,
    PromotionRowGenerator,
    ReasonRowGenerator,
    ShipModeRowGenerator,
    StoreRowGenerator,
    TimeDimRowGenerator,
    WarehouseRowGenerator,
    WebPageRowGenerator,
    WebSiteRowGenerator
);

// Implement factory for generators that emit sales and returns rows
impl_factory!(
    CatalogSalesRowGenerator,
    StoreSalesRowGenerator,
    WebSalesRowGenerator
);

/// Generate one planned table (one `--parts` chunk of one table) into
/// `base_location`, using up to `num_threads` threads.
///
/// A sales generator emits rows for its returns table too; each output keeps
/// only its own rows, the same way the Arrow generators produce them.
pub(super) async fn generate_table<F: RowFormat>(
    format: F,
    base_location: OutputLocation,
    planned: PlannedTable,
    num_threads: usize,
) -> io::Result<()> {
    macro_rules! generate {
        ($GENERATOR:ty) => {
            write_table::<F, $GENERATOR>(format, &base_location, planned, num_threads).await
        };
    }

    match planned.table {
        // Simple dimension tables
        Table::CallCenter => generate!(CallCenterRowGenerator),
        Table::CatalogPage => generate!(CatalogPageRowGenerator),
        Table::Customer => generate!(CustomerRowGenerator),
        Table::CustomerAddress => generate!(CustomerAddressRowGenerator),
        Table::CustomerDemographics => generate!(CustomerDemographicsRowGenerator),
        Table::DateDim => generate!(DateDimRowGenerator),
        Table::DbgenVersion => generate!(DbgenVersionRowGenerator),
        Table::HouseholdDemographics => generate!(HouseholdDemographicsRowGenerator),
        Table::IncomeBand => generate!(IncomeBandRowGenerator),
        Table::Inventory => generate!(InventoryRowGenerator),
        Table::Item => generate!(ItemRowGenerator),
        Table::Promotion => generate!(PromotionRowGenerator),
        Table::Reason => generate!(ReasonRowGenerator),
        Table::ShipMode => generate!(ShipModeRowGenerator),
        Table::Store => generate!(StoreRowGenerator),
        Table::TimeDim => generate!(TimeDimRowGenerator),
        Table::Warehouse => generate!(WarehouseRowGenerator),
        Table::WebPage => generate!(WebPageRowGenerator),
        Table::WebSite => generate!(WebSiteRowGenerator),

        // Sales tables and the returns tables their generator also emits
        Table::StoreSales | Table::StoreReturns => generate!(StoreSalesRowGenerator),
        Table::CatalogSales | Table::CatalogReturns => generate!(CatalogSalesRowGenerator),
        Table::WebSales | Table::WebReturns => generate!(WebSalesRowGenerator),

        // Source tables - skip
        _ => Ok(()),
    }
}

/// Generate the rows in `planned`.
///
/// Progress is counted in chunks; the totals are registered by
/// [`super::runner::plan_tables`]
async fn write_table<F, G>(
    format: F,
    base_location: &OutputLocation,
    planned: PlannedTable,
    num_threads: usize,
) -> io::Result<()>
where
    F: RowFormat,
    G: RowGeneratorFactory + Send + 'static,
{
    let PlannedTable {
        table,
        session,
        plan,
        progress,
    } = planned;

    let location = output_location_for_table(base_location, table, F::EXTENSION, &session)?;
    if location.skip_existing() {
        progress.increment(plan.chunk_count() as u64);
        progress.complete();
        return Ok(());
    }
    info!("Writing {location} using {num_threads} threads");

    let source_rows = session.get_scaling().get_row_count(table.source_table());
    let sources = plan.into_iter().map(move |range| RowSource::<F, G> {
        format: format.clone(),
        table,
        session: session.clone(),
        source_rows,
        range,
        generator: PhantomData,
    });

    match &location {
        OutputLocation::Stdout => {
            // Since generate_in_chunks already buffers, there is no need to
            // buffer again (aka don't use BufWriter here)
            let sink = WriterSink::new(io::stdout());
            generate_in_chunks(sink, sources, num_threads, progress.clone()).await?;
        }
        OutputLocation::File(path) => {
            generate_file(path, sources, num_threads, progress.clone()).await?;
        }
    }
    progress.complete();

    info!("Generated {location}");
    Ok(())
}

/// Generates the text for one chunk (a range of source rows) of one table.
struct RowSource<F, G> {
    format: F,
    table: Table,
    session: Session,
    source_rows: u64,
    /// The 1-based inclusive source rows of this chunk
    range: RangeInclusive<u64>,
    generator: PhantomData<G>,
}

impl<F, G> Source for RowSource<F, G>
where
    F: RowFormat,
    G: RowGeneratorFactory + Send + 'static,
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
            ..
        } = self;

        let mut rows = RowIter::new(G::create(), session, source_rows);
        rows.set_source_row_range(*range.start(), *range.end());

        format.write_rows(table, rows.filter(|row| row.table() == table), buffer)
    }
}
