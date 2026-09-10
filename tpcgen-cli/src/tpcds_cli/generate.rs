//! Driving the TPC-DS row generators, shared by the DAT and CSV outputs.
//!
//! Both outputs walk the generators identically — same row order, same seed
//! consumption, same sales/returns pairing — and differ only in how a table's
//! rows are written to a file. That difference is captured by [`TableOutput`]
//! and [`TableWriter`]; everything else lives here so the two formats cannot
//! drift apart.

use super::progress::TableProgress;
use log::info;
use std::io;
use std::path::PathBuf;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The output file for one table.
pub(super) trait TableWriter {
    /// Write one generated row.
    fn write_row(&mut self, row: &GeneratedRow) -> io::Result<()>;

    /// Flush and finalize the file, returning the path it was written to.
    fn finish(self) -> Result<PathBuf>;
}

/// An output format: creates the per-table writers.
pub(super) trait TableOutput {
    type Writer: TableWriter;

    /// Create the output file for `table`, ready to accept rows.
    fn create_writer(&self, table: Table, session: &Session) -> Result<Self::Writer>;
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

/// Generate one requested table into `output`.
pub(super) fn generate_table<O: TableOutput>(
    output: &O,
    table: Table,
    session: &Session,
    progress: TableProgress,
) -> Result<()> {
    match table {
        // Simple dimension tables
        Table::CallCenter => {
            generate_simple::<CallCenterRowGenerator, O>(output, table, session, progress)
        }
        Table::CatalogPage => {
            generate_simple::<CatalogPageRowGenerator, O>(output, table, session, progress)
        }
        Table::Customer => {
            generate_simple::<CustomerRowGenerator, O>(output, table, session, progress)
        }
        Table::CustomerAddress => {
            generate_simple::<CustomerAddressRowGenerator, O>(output, table, session, progress)
        }
        Table::CustomerDemographics => {
            generate_simple::<CustomerDemographicsRowGenerator, O>(output, table, session, progress)
        }
        Table::DateDim => {
            generate_simple::<DateDimRowGenerator, O>(output, table, session, progress)
        }
        Table::DbgenVersion => {
            generate_simple::<DbgenVersionRowGenerator, O>(output, table, session, progress)
        }
        Table::HouseholdDemographics => generate_simple::<HouseholdDemographicsRowGenerator, O>(
            output, table, session, progress,
        ),
        Table::IncomeBand => {
            generate_simple::<IncomeBandRowGenerator, O>(output, table, session, progress)
        }
        Table::Item => generate_simple::<ItemRowGenerator, O>(output, table, session, progress),
        Table::Promotion => {
            generate_simple::<PromotionRowGenerator, O>(output, table, session, progress)
        }
        Table::Reason => generate_simple::<ReasonRowGenerator, O>(output, table, session, progress),
        Table::ShipMode => {
            generate_simple::<ShipModeRowGenerator, O>(output, table, session, progress)
        }
        Table::Store => generate_simple::<StoreRowGenerator, O>(output, table, session, progress),
        Table::TimeDim => {
            generate_simple::<TimeDimRowGenerator, O>(output, table, session, progress)
        }
        Table::Warehouse => {
            generate_simple::<WarehouseRowGenerator, O>(output, table, session, progress)
        }
        Table::WebPage => {
            generate_simple::<WebPageRowGenerator, O>(output, table, session, progress)
        }
        Table::WebSite => {
            generate_simple::<WebSiteRowGenerator, O>(output, table, session, progress)
        }
        Table::Inventory => {
            generate_simple::<InventoryRowGenerator, O>(output, table, session, progress)
        }

        // Sales generators write their return tables at the same time.
        Table::StoreSales => generate_sales_and_returns::<StoreSalesRowGenerator, O>(
            output,
            Table::StoreSales,
            Table::StoreReturns,
            session,
            progress,
        ),
        Table::StoreReturns => Ok(()), // Generated with StoreSales
        Table::CatalogSales => generate_sales_and_returns::<CatalogSalesRowGenerator, O>(
            output,
            Table::CatalogSales,
            Table::CatalogReturns,
            session,
            progress,
        ),
        Table::CatalogReturns => Ok(()), // Generated with CatalogSales
        Table::WebSales => generate_sales_and_returns::<WebSalesRowGenerator, O>(
            output,
            Table::WebSales,
            Table::WebReturns,
            session,
            progress,
        ),
        Table::WebReturns => Ok(()), // Generated with WebSales

        // Source tables - skip
        _ => Ok(()),
    }
}

/// Generate a simple table (one row per row_number, no child tables)
fn generate_simple<G: RowGeneratorFactory, O: TableOutput>(
    output: &O,
    table: Table,
    session: &Session,
    progress: TableProgress,
) -> Result<()> {
    let TableProgress::Single(progress) = progress else {
        unreachable!("simple table must have one progress handle")
    };
    let mut generator = G::create();
    let row_count = session.get_scaling().get_row_count(table);

    let mut writer = output.create_writer(table, session)?;

    info!("Generating {}...", table.get_name());

    for row_number in 1..=row_count {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;

        for row in result.get_rows() {
            writer.write_row(row)?;
        }

        generator.consume_remaining_seeds_for_row();
        progress.increment(1);
    }

    let path = writer.finish()?;
    progress.complete();
    info!(
        "Generated {}: {} rows -> {}",
        table.get_name(),
        row_count,
        path.display()
    );

    Ok(())
}

/// Generate sales and returns tables in one pass.
///
/// Sales generators can emit rows for both output tables. Keeping them paired
/// preserves row advancement and seed consumption.
fn generate_sales_and_returns<G: RowGeneratorFactory, O: TableOutput>(
    output: &O,
    sales_table: Table,
    returns_table: Table,
    session: &Session,
    progress: TableProgress,
) -> Result<()> {
    let TableProgress::Paired {
        sales: sales_progress,
        returns: returns_progress,
    } = progress
    else {
        unreachable!("sales table must have sales and returns progress handles")
    };
    let mut generator = G::create();
    let source_row_count = session.get_scaling().get_row_count(sales_table);

    let mut sales_writer = output.create_writer(sales_table, session)?;
    let mut returns_writer = output.create_writer(returns_table, session)?;

    info!(
        "Generating {} + {}...",
        sales_table.get_name(),
        returns_table.get_name()
    );

    let mut sales_count = 0u64;
    let mut returns_count = 0u64;
    let mut row_number = 1u64;

    while row_number <= source_row_count {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            sales_writer.write_row(&rows[0])?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            returns_writer.write_row(&rows[1])?;
            returns_count += 1;
            returns_progress.increment(1);
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            row_number += 1;
            sales_progress.increment(1);
        }
    }

    let sales_path = sales_writer.finish()?;
    let returns_path = returns_writer.finish()?;
    sales_progress.complete();
    returns_progress.complete();

    info!(
        "Generated {} + {}: {} sales, {} returns -> {}, {}",
        sales_table.get_name(),
        returns_table.get_name(),
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}
