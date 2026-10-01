//! TPC-DS CSV output.
//!
//! Rows are formatted via the `tpcdsgen::csv` Display wrappers (the same
//! model as the TPC-H CSV output): one header line, then one line per row
//! with the same field values as the DAT output, joined by the delimiter
//! with no trailing separator. Free-text columns that can contain the
//! delimiter are double-quoted.
//!
//! Two deliberate differences from the DAT output, documented in more detail
//! on `tpcdsgen::csv`:
//!
//! * Output is UTF-8 in both compat modes, where the DAT output is ISO-8859-1
//!   in `CompatMode::Trino`. The values match as characters, not as bytes.
//! * Quoting is a fixed per-column property rather than quote-when-needed, so
//!   `--delimiter` is only safe for delimiters that no unquoted column
//!   contains (`,`, `|`, tab, `;`).

use crate::output_location::OutputLocation;
use crate::progress::ProgressTracker;
use crate::tpcds_cli::generate::{generate_table, RowFormat};
use crate::tpcds_cli::plan::ChunkFormat;
use crate::tpcds_cli::runner::{plan_tables, run_plans};
use std::fmt::Display;
use std::io::{self, Write};
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::csv::*;
use tpcdsgen::row::*;

/// CSV output generator.
#[derive(Debug, Clone)]
pub(super) struct Csv {
    base_location: OutputLocation,
    pub(super) delimiter: char,
    chunk_size_bytes: i64,
}

impl Csv {
    pub(super) fn new(
        base_location: OutputLocation,
        delimiter: char,
        chunk_size_bytes: i64,
    ) -> Self {
        Self {
            base_location,
            delimiter,
            chunk_size_bytes,
        }
    }

    /// Generate the given TPC-DS tables as CSV files.
    pub(super) async fn generate_tables(
        &self,
        table_sessions: Vec<(Table, Session)>,
        num_threads: usize,
        progress: Arc<dyn ProgressTracker>,
    ) -> io::Result<()> {
        // Check every header up front: a CSV file is not valid without one,
        // and `write_header` cannot report an error once generation starts.
        for (table, _) in &table_sessions {
            if csv_header(*table, self.delimiter).is_none() {
                return Err(io::Error::other(format!(
                    "table {} has no CSV output",
                    table.get_name()
                )));
            }
        }

        let work = plan_tables(
            table_sessions,
            self.chunk_size_bytes,
            ChunkFormat::Csv,
            &progress,
        );
        progress.start();

        let this = self.clone();
        run_plans(work, num_threads, move |planned, num_threads| {
            let format = this.clone();
            let base_location = this.base_location.clone();
            async move { generate_table(format, base_location, planned, num_threads).await }
        })
        .await
    }
}

/// A row type with a CSV rendering.
pub(super) trait CsvRow {
    type Csv<'a>: Display
    where
        Self: 'a;

    fn csv(&self, delimiter: char) -> Self::Csv<'_>;
}

macro_rules! impl_csv_row {
    ($($row:ident => $csv:ident),* $(,)?) => {
        $(
            impl CsvRow for $row {
                type Csv<'a> = $csv<'a>;

                fn csv(&self, delimiter: char) -> $csv<'_> {
                    $csv::with_delimiter(self, delimiter)
                }
            }
        )*
    };
}

impl_csv_row!(
    CallCenterRow => CallCenterCsv,
    CatalogPageRow => CatalogPageCsv,
    CatalogReturnsRow => CatalogReturnsCsv,
    CatalogSalesRow => CatalogSalesCsv,
    CustomerRow => CustomerCsv,
    CustomerAddressRow => CustomerAddressCsv,
    CustomerDemographicsRow => CustomerDemographicsCsv,
    DateDimRow => DateDimCsv,
    DbgenVersionRow => DbgenVersionCsv,
    HouseholdDemographicsRow => HouseholdDemographicsCsv,
    IncomeBandRow => IncomeBandCsv,
    InventoryRow => InventoryCsv,
    ItemRow => ItemCsv,
    PromotionRow => PromotionCsv,
    ReasonRow => ReasonCsv,
    ShipModeRow => ShipModeCsv,
    StoreRow => StoreCsv,
    StoreReturnsRow => StoreReturnsCsv,
    StoreSalesRow => StoreSalesCsv,
    TimeDimRow => TimeDimCsv,
    WarehouseRow => WarehouseCsv,
    WebPageRow => WebPageCsv,
    WebReturnsRow => WebReturnsCsv,
    WebSalesRow => WebSalesCsv,
    WebSiteRow => WebSiteCsv,
);

impl<R: CsvRow> RowFormat<R> for Csv {
    const EXTENSION: &'static str = "csv";

    fn write_header(&self, table: Table, mut buffer: Vec<u8>) -> Vec<u8> {
        // Checked by `generate_tables` before any generation starts.
        let header = csv_header(table, self.delimiter)
            .unwrap_or_else(|| panic!("table {} has no CSV output", table.get_name()));
        writeln!(buffer, "{header}").expect("writing to memory cannot fail");
        buffer
    }

    fn write_rows<I>(&self, _table: Table, rows: I, mut buffer: Vec<u8>) -> Vec<u8>
    where
        I: Iterator<Item = R>,
    {
        for row in rows {
            writeln!(buffer, "{}", row.csv(self.delimiter)).expect("writing to memory cannot fail");
        }
        buffer
    }
}
