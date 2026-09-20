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

//! TPC-DS DAT output.
//!
//! Generates TPC-DS benchmark data with byte-for-byte compatibility with the Java reference.

use super::generate::{generate_table, RowFormat};
use super::plan::ChunkFormat;
use super::runner::{plan_tables, run_plans};
use crate::progress::ProgressTracker;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use tpcdsgen::config::{CompatMode, Session, Table};
use tpcdsgen::error::InvalidOptionError;
use tpcdsgen::output::DatWriter;
use tpcdsgen::row::GeneratedRow;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DAT output generator.
#[derive(Debug, Clone)]
pub(super) struct Dat {
    /// Where to write the output
    output_dir: PathBuf,
    /// Which reference implementation to match.
    compat_mode: CompatMode,
    /// Target size of each generated buffer
    chunk_size_bytes: i64,
}

impl Dat {
    pub(super) fn new(
        output_dir: PathBuf,
        compat_mode: CompatMode,
        chunk_size_bytes: i64,
    ) -> Result<Self> {
        if output_dir.as_os_str().is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                "",
                "Directory cannot be empty",
            )
            .into());
        }
        Ok(Self {
            output_dir,
            compat_mode,
            chunk_size_bytes,
        })
    }

    /// Generate the given TPC-DS tables as DAT files.
    pub(super) async fn generate_tables(
        &self,
        table_sessions: Vec<(Table, Session)>,
        num_threads: usize,
        progress: Arc<dyn ProgressTracker>,
    ) -> io::Result<()> {
        let work = plan_tables(
            table_sessions,
            self.chunk_size_bytes,
            ChunkFormat::Dat,
            &progress,
        );
        progress.start();

        let this = self.clone();
        run_plans(work, num_threads, move |planned, num_threads| {
            let format = this.clone();
            let output_dir = this.output_dir.clone();
            async move { generate_table(format, output_dir, planned, num_threads).await }
        })
        .await
    }
}

impl RowFormat for Dat {
    const EXTENSION: &'static str = "dat";

    /// DAT output has no header.
    fn write_header(&self, _table: Table, buffer: Vec<u8>) -> Vec<u8> {
        buffer
    }

    fn write_rows<I>(&self, _table: Table, rows: I, mut buffer: Vec<u8>) -> Vec<u8>
    where
        I: Iterator<Item = GeneratedRow>,
    {
        let mut writer = DatWriter::new(&mut buffer, self.compat_mode);
        for row in rows {
            // Writing to memory cannot fail, and every generated value is
            // representable in the output encoding (the distributions the
            // values come from are themselves ISO-8859-1).
            writer
                .write_display_row(&row)
                .expect("DAT rows are always writable to memory");
        }
        writer
            .flush()
            .expect("DAT rows are always writable to memory");
        drop(writer);

        buffer
    }
}
