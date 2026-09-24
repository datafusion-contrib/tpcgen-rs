//! Statistics reporter for data generation

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
        info!(
            "Created {} in {duration:.2?} ({}/sec)",
            format_bytes(self.num_bytes as u64),
            format_bytes(bytes_per_second)
        );
        debug!(
            "Wrote {} bytes in {} {}  {mb_per_chunk:.02} MB/{}",
            self.num_bytes, self.num_chunks, self.chunk_label, self.chunk_label
        );
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.2} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn readable_byte_sizes() {
        for (bytes, expected) in [
            (0, "0 B"),
            (389, "389 B"),
            (1023, "1023 B"),
            (1024, "1.00 KiB"),
            (1536, "1.50 KiB"),
            (1 << 20, "1.00 MiB"),
            (1 << 30, "1.00 GiB"),
            (1 << 40, "1.00 TiB"),
            (1 << 50, "1.00 PiB"),
            (u64::MAX, "16.00 EiB"),
        ] {
            assert_eq!(format_bytes(bytes), expected);
        }
    }
}
