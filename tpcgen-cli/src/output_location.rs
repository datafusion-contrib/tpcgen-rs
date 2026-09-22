//! [`OutputLocation`]: where generated data is written.

use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Where a generated table (or one part of one) is written
#[derive(Debug, Clone, PartialEq)]
pub enum OutputLocation {
    /// Output to a file
    File(PathBuf),
    /// Output to stdout
    Stdout,
}

impl Display for OutputLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputLocation::File(path) => {
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
