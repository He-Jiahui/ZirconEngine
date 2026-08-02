use std::path::Path;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, io};

impl RuntimeSessionArchive {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, RuntimeSessionArchiveError> {
        io::load_from_path(path)
    }

    pub fn load_or_empty_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)
    }
}
