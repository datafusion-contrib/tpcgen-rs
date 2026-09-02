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

use crate::progress::ProgressTracker;
use crate::temp_path::inprogress_path;
use crate::tpcds_cli::generate::{generate_table, part_aware_path, TableOutput, TableWriter};
use crate::tpcds_cli::progress::{register_table, TableProgress};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::csv::{csv_header, GeneratedRowCsv};
use tpcdsgen::row::GeneratedRow;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// CSV output generator.
#[derive(Debug, Clone)]
pub(super) struct Csv {
    output_dir: PathBuf,
    delimiter: char,
}

impl Csv {
    pub(super) fn new(output_dir: PathBuf, delimiter: char) -> Self {
        Self {
            output_dir,
            delimiter,
        }
    }

    pub(super) fn register_table(
        &self,
        table: Table,
        session: &Session,
        progress: Arc<dyn ProgressTracker>,
    ) -> TableProgress {
        register_table(table, session, progress)
    }

    /// Generate one TPC-DS table as a CSV file.
    pub(super) fn generate_table(
        &self,
        table: Table,
        session: &Session,
        progress: TableProgress,
    ) -> Result<()> {
        generate_table(self, table, session, progress)
    }
}

impl TableOutput for Csv {
    type Writer = CsvTableFile;

    /// Create the CSV file for `table` (written to a temporary `.inprogress`
    /// path until finished) and write the header line.
    fn create_writer(&self, table: Table, session: &Session) -> Result<Self::Writer> {
        let path = part_aware_path(&self.output_dir, table, "csv", session)?;
        let header = csv_header(table, self.delimiter)
            .ok_or_else(|| format!("table {} has no CSV output", table.get_name()))?;
        CsvTableFile::create(path, &header, self.delimiter)
    }
}

/// One in-progress CSV output file: rows are written to `<table>.csv.inprogress`,
/// which is renamed to `<table>.csv` on `finish`.
pub(super) struct CsvTableFile {
    writer: BufWriter<File>,
    temp_path: PathBuf,
    path: PathBuf,
    delimiter: char,
}

impl CsvTableFile {
    fn create(path: PathBuf, header: &str, delimiter: char) -> Result<Self> {
        let temp_path = inprogress_path(&path);
        let file = File::create(&temp_path)
            .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
        let mut writer = BufWriter::with_capacity(32 * 1024 * 1024, file);
        writeln!(writer, "{header}")?;
        Ok(Self {
            writer,
            temp_path,
            path,
            delimiter,
        })
    }
}

impl TableWriter for CsvTableFile {
    fn write_row(&mut self, row: &GeneratedRow) -> io::Result<()> {
        writeln!(
            self.writer,
            "{}",
            GeneratedRowCsv::with_delimiter(row, self.delimiter)
        )
    }

    /// Flush and rename the temporary file into place, returning the final path.
    fn finish(self) -> Result<PathBuf> {
        // Close the file before renaming: Windows can refuse to rename a file
        // that is still open.
        let file = self.writer.into_inner().map_err(|err| {
            io::Error::other(format!("Failed to write {:?}: {err}", self.temp_path))
        })?;
        drop(file);
        std::fs::rename(&self.temp_path, &self.path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {:?} to {:?} file: {err}",
                self.temp_path, self.path
            ))
        })?;
        Ok(self.path)
    }
}
