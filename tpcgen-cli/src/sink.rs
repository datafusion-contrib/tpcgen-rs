//! [`WriterSink`]: a [`Sink`] that writes buffers to a [`Write`].
//!
//! Shared by the TPC-H and TPC-DS text (TBL/CSV/DAT) outputs, which all
//! generate buffers in parallel with [`generate_in_chunks`] and hand them to
//! a single writer task.
//!
//! [`generate_in_chunks`]: crate::generate::generate_in_chunks

use crate::generate::Sink;
use crate::parquet::IntoSize;
use crate::statistics::WriteStatistics;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Stdout, Write};

/// Wrapper around a buffer writer that counts the number of buffers and bytes written
pub struct WriterSink<W: Write> {
    statistics: WriteStatistics,
    inner: W,
}

impl<W: Write> WriterSink<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            statistics: WriteStatistics::new("buffers"),
        }
    }
}

impl<W: Write + Send> Sink for WriterSink<W> {
    fn sink(&mut self, buffer: &[u8]) -> Result<(), io::Error> {
        self.statistics.increment_chunks(1);
        self.statistics.increment_bytes(buffer.len());
        self.inner.write_all(buffer)
    }

    fn flush(mut self) -> Result<(), io::Error> {
        self.inner.flush()
    }
}

impl IntoSize for BufWriter<Stdout> {
    fn into_size(self) -> Result<usize, io::Error> {
        // we can't get the size of stdout, so just return 0
        Ok(0)
    }
}

impl IntoSize for BufWriter<File> {
    fn into_size(self) -> Result<usize, io::Error> {
        let file = self.into_inner()?;
        let metadata = file.metadata()?;
        Ok(metadata.len() as usize)
    }
}
