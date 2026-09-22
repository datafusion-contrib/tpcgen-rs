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
use crate::output_location::OutputLocation;
use crate::progress::ProgressTracker;
use std::io;
use std::sync::Arc;

use tpcdsgen::config::{CompatMode, Session, Table};
use tpcdsgen::output::DatWriter;
use tpcdsgen::row::GeneratedRow;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DAT output generator.
#[derive(Debug, Clone)]
pub(super) struct Dat {
    /// Where to write the output
    base_location: OutputLocation,
    /// Which reference implementation to match.
    compat_mode: CompatMode,
    /// Target size of each generated buffer
    chunk_size_bytes: i64,
}

impl Dat {
    pub(super) fn new(
        base_location: OutputLocation,
        compat_mode: CompatMode,
        chunk_size_bytes: i64,
    ) -> Result<Self> {
        Ok(Self {
            base_location,
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
            let base_location = this.base_location.clone();
            async move { generate_table(format, base_location, planned, num_threads).await }
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
