//! [`WriterSink`]: a [`Sink`] that writes buffers to a [`Write`].

use crate::generate::Sink;
use crate::statistics::WriteStatistics;
use std::io;
use std::io::Write;

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
