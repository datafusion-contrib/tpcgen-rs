//! [`OutputLocation`]: where generated data is written.

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Where generated data is written: the filesystem, or stdout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputLocation {
    /// Output to a file in the specified directory
    File {
        path: PathBuf,
        /// Whether existing files at or under `path` are overwritten
        overwrite: bool,
    },
    /// Output to stdout
    Stdout,
}

impl OutputLocation {
    /// Return the location selected on the command line: stdout when
    /// `--stdout` was given, and `output_dir` otherwise.
    pub fn new(stdout: bool, output_dir: PathBuf, overwrite: bool) -> Self {
        if stdout {
            Self::Stdout
        } else {
            Self::File {
                path: output_dir,
                overwrite,
            }
        }
    }

    /// Return the location of `path` within this output.
    ///
    /// [`Self::Stdout`] has no path to join onto, so it is returned unchanged.
    pub fn join(&self, path: impl AsRef<Path>) -> Self {
        match self {
            Self::File {
                path: base,
                overwrite,
            } => Self::File {
                path: base.join(path),
                overwrite: *overwrite,
            },
            Self::Stdout => Self::Stdout,
        }
    }

    /// Return true if this location is a path with nothing in it (`-o ""`),
    /// which names no directory.
    pub fn is_empty_dir(&self) -> bool {
        match self {
            Self::File { path, .. } => path.as_os_str().is_empty(),
            Self::Stdout => false,
        }
    }

    /// Create this location's directory, and any missing parents, if it does
    /// not already exist.
    ///
    /// Does nothing for [`Self::Stdout`]
    pub fn create_dir_all(&self) -> io::Result<()> {
        let Self::File { path: dir, .. } = self else {
            return Ok(());
        };
        std::fs::create_dir_all(dir).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Error creating directory {}: {e}", dir.display()),
            )
        })
    }

    /// Write `output` to this location.
    ///
    /// Files are written to `<path>.inprogress` and renamed on success. The
    /// temporary file is removed if writing fails or is cancelled. Existing
    /// files are skipped unless `overwrite` is set, returning `Ok(false)`.
    pub(crate) async fn write<O: WriteOutput>(&self, output: O) -> io::Result<bool> {
        let (path, overwrite) = match self {
            Self::Stdout => {
                output.write_to(io::stdout()).await?;
                return Ok(true);
            }
            Self::File { path, overwrite } => (path, *overwrite),
        };
        if !overwrite && path.exists() {
            log::warn!("{} already exists, skipping generation", path.display());
            return Ok(false);
        }

        let in_progress = InProgressFile::new(path);
        output.write_to(in_progress.create()?).await?;
        in_progress.finish()?;
        Ok(true)
    }
}

/// The `<path>.inprogress` file that output is written to before it is
/// renamed to `<path>`.
///
/// If this is dropped before [`Self::finish`] (because writing failed or was
/// cancelled), the `.inprogress` file is deleted.
struct InProgressFile<'a> {
    path: &'a Path,
    temp_path: PathBuf,
}

impl<'a> InProgressFile<'a> {
    fn new(path: &'a Path) -> Self {
        // Append to the full file name (unlike `with_extension`), so
        // `lineitem.1.tbl` and `lineitem.2.tbl` stay distinct
        let mut temp_path = path.as_os_str().to_owned();
        temp_path.push(".inprogress");
        Self {
            path,
            temp_path: temp_path.into(),
        }
    }

    /// Create `<path>.inprogress` for writing.
    fn create(&self) -> io::Result<File> {
        File::create(&self.temp_path).map_err(|err| {
            io::Error::other(format!("Failed to create {:?}: {err}", self.temp_path))
        })
    }

    /// Rename `<path>.inprogress` to `<path>`.
    fn finish(self) -> io::Result<()> {
        std::fs::rename(&self.temp_path, self.path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {:?} to {:?} file: {err}",
                self.temp_path, self.path
            ))
        })
    }
}

impl Drop for InProgressFile<'_> {
    fn drop(&mut self) {
        // After a successful `finish` there is nothing left to delete
        let _ = std::fs::remove_file(&self.temp_path);
    }
}

/// Something that can write generated output to any [`Write`]
///
/// For example, this is implemented for text and Parquet output.
/// `write_to` is generic over the writer, so each output is compiled
/// separately for stdout and for files.
pub(crate) trait WriteOutput {
    /// Generate the output into `writer`
    async fn write_to<W: Write + Send + 'static>(self, writer: W) -> io::Result<()>;
}

impl Display for OutputLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputLocation::File { path, .. } => {
                let Some(file) = path.file_name() else {
                    return write!(f, "{}", path.display());
                };
                // Display the file name only, not the full path
                write!(f, "{}", file.to_string_lossy())
            }
            OutputLocation::Stdout => write!(f, "Stdout"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    struct NeverFinishes;

    impl WriteOutput for NeverFinishes {
        async fn write_to<W: Write + Send + 'static>(self, _writer: W) -> io::Result<()> {
            std::future::pending().await
        }
    }

    #[test]
    fn cancelled_write_removes_inprogress_file() {
        let dir = tempfile::tempdir().unwrap();
        let location = OutputLocation::File {
            path: dir.path().join("region.tbl"),
            overwrite: false,
        };

        // Start the write, then drop it before it finishes
        assert!(location.write(NeverFinishes).now_or_never().is_none());

        assert!(!dir.path().join("region.tbl.inprogress").exists());
    }
}
