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
//!
//! Generation is parallel: see [`super::generate`] for how the row generators
//! are driven, and [`super::runner`] for how tables are planned and scheduled.

use super::generate::{generate_table, RowFormat};
use super::plan::ChunkFormat;
use super::runner::{plan_tables, run_plans};
use crate::progress::ProgressTracker;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::csv::{csv_header, GeneratedRowCsv};
use tpcdsgen::row::GeneratedRow;

/// CSV output generator.
#[derive(Debug, Clone)]
pub(super) struct Csv {
    output_dir: PathBuf,
    delimiter: char,
    /// Target size of each generated buffer
    chunk_bytes: i64,
}

impl Csv {
    pub(super) fn new(output_dir: PathBuf, delimiter: char, chunk_bytes: i64) -> Self {
        Self {
            output_dir,
            delimiter,
            chunk_bytes,
        }
    }

    /// Generate the given TPC-DS tables as CSV files.
    pub(super) async fn generate_tables(
        &self,
        table_sessions: Vec<(Table, Session)>,
        num_threads: usize,
        progress: Arc<dyn ProgressTracker>,
    ) -> io::Result<()> {
        // Every table must have a header before any file is created: unlike
        // DAT, a CSV file is not valid without one, and `write_header` cannot
        // report an error once generation has started.
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
            self.chunk_bytes,
            ChunkFormat::Csv,
            &progress,
        );
        progress.start();

        let this = self.clone();
        run_plans(work, num_threads, move |planned, num_threads| {
            let this = this.clone();
            async move {
                generate_table(this.clone(), this.output_dir.clone(), planned, num_threads).await
            }
        })
        .await
    }
}

impl RowFormat for Csv {
    const EXTENSION: &'static str = "csv";

    fn write_header(&self, table: Table, mut buffer: Vec<u8>) -> Vec<u8> {
        // Validated by `generate_tables` before any generation starts.
        let header = csv_header(table, self.delimiter)
            .unwrap_or_else(|| panic!("table {} has no CSV output", table.get_name()));
        writeln!(buffer, "{header}").expect("writing to memory cannot fail");
        buffer
    }

    fn write_rows<I>(&self, _table: Table, rows: I, mut buffer: Vec<u8>) -> Vec<u8>
    where
        I: Iterator<Item = GeneratedRow>,
    {
        for row in rows {
            writeln!(
                buffer,
                "{}",
                GeneratedRowCsv::with_delimiter(&row, self.delimiter)
            )
            .expect("writing to memory cannot fail");
        }
        buffer
    }
}
