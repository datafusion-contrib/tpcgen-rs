//! Statistics reporter for data generation, shared by TPC-H and TPC-DS.

use log::{debug, info};
use std::time::Instant;

/// Statistics for writing data to a file
///
/// Reports the statistics on drop
#[derive(Clone, Debug)]
pub struct WriteStatistics {
    /// Time at which the writer was created
    start: Instant,
    /// User defined "chunks" (e.g. buffers or row_groups)
    num_chunks: usize,
    chunk_label: String,
    /// total bytes written
    num_bytes: usize,
}

impl WriteStatistics {
    /// Create a new statistics reporter
    pub fn new(chunk_label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            num_chunks: 0,
            chunk_label: chunk_label.into(),
            num_bytes: 0,
        }
    }

    /// Increment chunk count
    pub fn increment_chunks(&mut self, num_chunks: usize) {
        self.num_chunks += num_chunks;
    }

    /// Increment byte count
    pub fn increment_bytes(&mut self, num_bytes: usize) {
        self.num_bytes += num_bytes;
    }
}

impl Drop for WriteStatistics {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let mb_per_chunk = self.num_bytes as f64 / (1024.0 * 1024.0) / self.num_chunks as f64;
        let bytes_per_second = (self.num_bytes as f64 / duration.as_secs_f64()) as u64;
        let gb_per_second = bytes_per_second as f64 / (1024.0 * 1024.0 * 1024.0);
        let gb = self.num_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        info!("Created {gb:.02} GB in {duration:?} ({gb_per_second:.02} GB/sec)");
        debug!(
            "Wrote {} bytes in {} {}  {mb_per_chunk:.02} MB/{}",
            self.num_bytes, self.num_chunks, self.chunk_label, self.chunk_label
        );
    }
}
