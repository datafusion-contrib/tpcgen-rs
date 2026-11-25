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

//! Output utilities for TPC-DS data generation
//!
//! The Java (Trino) implementation reads distribution files as ISO-8859-1
//! (Latin-1) and writes output files as ISO-8859-1 (see TableGenerator.java
//! line 80). The C `dsdgen` outputs UTF-8.
//!
//! [`DatWriter`] selects the right behavior based on [`CompatMode`].

use std::fmt;
use std::io::{self, Write};

use crate::config::CompatMode;

/// Converts a UTF-8 string to ISO-8859-1 bytes.
///
/// This is the inverse of the conversion done in file_loader.rs when reading
/// distribution files. Characters must be in the range U+0000-U+00FF.
///
/// # Errors
/// Returns an error if any character is outside the ISO-8859-1 range (U+0000-U+00FF).
pub fn to_iso_8859_1(s: &str) -> io::Result<Vec<u8>> {
    s.chars()
        .map(|c| {
            let code = c as u32;
            if code > 255 {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Character '{}' (U+{:04X}) is outside ISO-8859-1 range",
                        c, code
                    ),
                ))
            } else {
                Ok(code as u8)
            }
        })
        .collect()
}

/// Buffered DAT row writer.
///
/// Rows are formatted into an in-memory UTF-8 buffer which is encoded and
/// flushed to the inner writer one large chunk at a time. Encoding per chunk
/// instead of per field keeps the hot path to plain byte appends: the common
/// all-ASCII chunk is written through unchanged in both compat modes (ASCII
/// bytes are identical in UTF-8 and ISO-8859-1), and only chunks that
/// actually contain non-ASCII characters pay for ISO-8859-1 conversion in
/// Trino mode.
pub struct DatWriter<W: Write> {
    inner: W,
    compat_mode: CompatMode,
    /// Pending formatted rows (UTF-8).
    buffer: Vec<u8>,
    /// Keep the separator that follows the last field of each row.
    trailing_delimiter: bool,
}

impl<W: Write> DatWriter<W> {
    /// Flush the pending buffer once it grows past this size: large enough to
    /// amortize the encoding scan and write call, small enough to stay in L2.
    const FLUSH_THRESHOLD: usize = 64 * 1024;

    pub fn new(inner: W, compat_mode: CompatMode) -> Self {
        Self {
            inner,
            compat_mode,
            buffer: Vec::with_capacity(Self::FLUSH_THRESHOLD + 1024),
            trailing_delimiter: true,
        }
    }

    /// Choose whether each row keeps the separator after its last field
    /// (default: `true`, matching the reference generators' default output).
    ///
    /// With `false`, exactly one final `|` is removed from every row, matching
    /// `dsdgen -terminate N`. A NULL last field is empty, so the row `a||`
    /// becomes `a|`.
    pub fn with_trailing_delimiter(mut self, trailing_delimiter: bool) -> Self {
        self.trailing_delimiter = trailing_delimiter;
        self
    }

    /// Write one row whose `Display` impl emits the DAT line (fields joined
    /// by the separator, with a trailing separator); appends the newline.
    ///
    /// The trailing separator is dropped if
    /// [`with_trailing_delimiter(false)`](Self::with_trailing_delimiter) was set.
    pub fn write_display_row(&mut self, row: &impl fmt::Display) -> io::Result<()> {
        write!(self.buffer, "{row}")?;
        if !self.trailing_delimiter && self.buffer.last() == Some(&b'|') {
            self.buffer.pop();
        }
        self.buffer.push(b'\n');
        self.maybe_flush()
    }

    /// Encode and flush the pending buffer if it has grown past the threshold.
    fn maybe_flush(&mut self) -> io::Result<()> {
        if self.buffer.len() >= Self::FLUSH_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        match self.compat_mode {
            CompatMode::C => self.inner.write_all(&self.buffer)?,
            CompatMode::Trino => {
                if self.buffer.is_ascii() {
                    self.inner.write_all(&self.buffer)?;
                } else {
                    let s = std::str::from_utf8(&self.buffer)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    self.inner.write_all(&to_iso_8859_1(s)?)?;
                }
            }
        }
        self.buffer.clear();
        Ok(())
    }

    /// Flush all pending rows and the inner writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.flush_buffer()?;
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_iso_8859_1_ascii() {
        let result = to_iso_8859_1("Hello").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_to_iso_8859_1_latin1() {
        // Ô is U+00D4, which should become byte 0xD4
        let result = to_iso_8859_1("CÔTE D'IVOIRE").unwrap();
        assert_eq!(result[1], 0xD4); // The Ô character
        assert_eq!(result.len(), 13); // One byte per character
    }

    #[test]
    fn test_to_iso_8859_1_out_of_range() {
        // Euro sign € is U+20AC, outside ISO-8859-1 range
        let result = to_iso_8859_1("€100");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("outside ISO-8859-1 range"));
    }

    struct TestRow;

    impl std::fmt::Display for TestRow {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "1|CÔTE|2.50|")
        }
    }

    #[test]
    fn test_dat_writer_buffers_until_flush() {
        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::Trino);
        writer.write_display_row(&TestRow).unwrap();
        // Small rows stay buffered until an explicit flush.
        assert!(!writer.buffer.is_empty());
        writer.flush().unwrap();
        // Ô (U+00D4) must come out as the single ISO-8859-1 byte 0xD4.
        assert_eq!(out, b"1|C\xD4TE|2.50|\n");
    }

    #[test]
    fn test_dat_writer_utf8_mode_passes_through() {
        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::C);
        writer.write_display_row(&TestRow).unwrap();
        writer.flush().unwrap();
        assert_eq!(out, "1|CÔTE|2.50|\n".as_bytes());
    }

    #[test]
    fn test_dat_writer_no_trailing_delimiter_removes_one_pipe() {
        struct NullLastRow;
        impl std::fmt::Display for NullLastRow {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a||")
            }
        }

        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::C).with_trailing_delimiter(false);
        writer.write_display_row(&TestRow).unwrap();
        writer.write_display_row(&NullLastRow).unwrap();
        writer.flush().unwrap();
        assert_eq!(out, "1|CÔTE|2.50\na|\n".as_bytes());
    }

    #[test]
    fn test_dat_writer_rejects_non_latin1_in_trino_mode() {
        struct EuroRow;
        impl std::fmt::Display for EuroRow {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "€100|")
            }
        }

        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::Trino);
        writer.write_display_row(&EuroRow).unwrap();
        let err = writer.flush().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
