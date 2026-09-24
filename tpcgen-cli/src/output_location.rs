//! [`OutputLocation`]: where generated data is written.

use std::fmt::{Display, Formatter};
use std::io;
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

    /// Return true if generation should be skipped because this location's
    /// file already exists, and log a warning saying so.
    ///
    /// Always false when `overwrite` is set, and for [`Self::Stdout`].
    pub(crate) fn skip_existing(&self) -> bool {
        let Self::File { path, overwrite } = self else {
            return false;
        };
        if *overwrite || !path.exists() {
            return false;
        }
        log::warn!("{} already exists, skipping generation", path.display());
        true
    }

    /// Return true if this location is a path with nothing in it (`-o ""`),
    /// which names no directory.
    pub fn is_empty_dir(&self) -> bool {
        match self {
            Self::File { path, .. } => path.as_os_str().is_empty(),
            Self::Stdout => false,
        }
    }
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
