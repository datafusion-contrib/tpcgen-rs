//! Naming for the temporary files used while generating output.

use std::path::{Path, PathBuf};

/// Returns the temporary path used while `path` is being written.
///
/// The suffix is *appended* to the full file name, so `lineitem.csv` becomes
/// `lineitem.csv.inprogress` (rather than `lineitem.inprogress`, which is what
/// [`Path::with_extension`] would produce). Keeping the original extension
/// makes partially written files obvious, and keeps per-part outputs such as
/// `lineitem.1.tbl` and `lineitem.2.tbl` distinct while in progress.
pub(crate) fn inprogress_path(path: &Path) -> PathBuf {
    let mut temp_path = path.to_path_buf().into_os_string();
    temp_path.push(".inprogress");
    PathBuf::from(temp_path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn appends_to_full_file_name() {
        assert_eq!(
            inprogress_path(Path::new("/tmp/out/lineitem.csv")),
            PathBuf::from("/tmp/out/lineitem.csv.inprogress")
        );
    }

    #[test]
    fn keeps_parts_distinct() {
        assert_eq!(
            inprogress_path(Path::new("/tmp/out/lineitem.1.tbl")),
            PathBuf::from("/tmp/out/lineitem.1.tbl.inprogress")
        );
        assert_eq!(
            inprogress_path(Path::new("/tmp/out/lineitem.2.tbl")),
            PathBuf::from("/tmp/out/lineitem.2.tbl.inprogress")
        );
    }
}
