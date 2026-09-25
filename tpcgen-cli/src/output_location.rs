//! [`OutputLocation`]: where generated data is written.

use crate::temp_path::inprogress_path;
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
    /// Files are written to `<path>.inprogress` and renamed on success. Existing
    /// files are skipped unless `overwrite` is set, returning `Ok(false)`.
    /// Calls `on_start` after the skip check, before attempting to write.
    pub(crate) async fn write<O: WriteOutput>(
        &self,
        output: O,
        on_start: impl FnOnce(),
    ) -> io::Result<bool> {
        let (path, overwrite) = match self {
            Self::Stdout => {
                on_start();
                output.write_to(io::stdout()).await?;
                return Ok(true);
            }
            Self::File { path, overwrite } => (path, *overwrite),
        };
        if !overwrite && path.exists() {
            log::warn!("{} already exists, skipping generation", path.display());
            return Ok(false);
        }

        on_start();
        let temp_path = inprogress_path(path);
        let file = File::create(&temp_path)
            .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
        output.write_to(file).await?;
        std::fs::rename(&temp_path, path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {temp_path:?} to {path:?} file: {err}"
            ))
        })?;
        Ok(true)
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
