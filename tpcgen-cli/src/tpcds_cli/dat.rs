/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! TPC-DS Data Generator - Rust Implementation
//!
//! Generates TPC-DS benchmark data with byte-for-byte compatibility with the Java reference.

use super::generate::{generate_table, part_aware_path, TableOutput, TableWriter};
use super::progress::{register_table, TableProgress};
use crate::progress::ProgressTracker;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use tpcdsgen::config::{Session, Table};
use tpcdsgen::error::InvalidOptionError;
use tpcdsgen::output::DatWriter;
use tpcdsgen::row::GeneratedRow;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DAT output generator.
///
/// Output is always the reference DAT format: `|`-separated fields with a
/// trailing separator, one row per line, written to `<table>.dat` files via
/// each row type's `Display` impl.
#[derive(Debug, Clone)]
pub(super) struct Dat {
    output_dir: PathBuf,
}

impl Dat {
    pub(super) fn new(output_dir: PathBuf) -> Result<Self> {
        if output_dir.as_os_str().is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                "",
                "Directory cannot be empty",
            )
            .into());
        }
        Ok(Self { output_dir })
    }

    pub(super) fn register_table(
        &self,
        table: Table,
        session: &Session,
        progress: Arc<dyn ProgressTracker>,
    ) -> TableProgress {
        register_table(table, session, progress)
    }

    pub(super) fn generate_table(
        &self,
        table: Table,
        session: &Session,
        progress: TableProgress,
    ) -> Result<()> {
        generate_table(self, table, session, progress)
    }
}

impl TableOutput for Dat {
    type Writer = DatTableWriter;

    fn create_writer(&self, table: Table, session: &Session) -> Result<Self::Writer> {
        let path = part_aware_path(&self.output_dir, table, "dat", session)?;
        let writer = DatWriter::new(File::create(&path)?, session.get_compat_mode());
        Ok(DatTableWriter { writer, path })
    }
}

/// One DAT output file. Rows are written straight to `<table>.dat`.
pub(super) struct DatTableWriter {
    writer: DatWriter<File>,
    path: PathBuf,
}

impl TableWriter for DatTableWriter {
    fn write_row(&mut self, row: &GeneratedRow) -> io::Result<()> {
        self.writer.write_display_row(row)
    }

    fn finish(mut self) -> Result<PathBuf> {
        self.writer.flush()?;
        Ok(self.path)
    }
}
