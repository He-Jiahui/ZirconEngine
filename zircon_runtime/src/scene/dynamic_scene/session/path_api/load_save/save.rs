use std::path::Path;

use super::super::super::{io, RuntimeSessionArchive, RuntimeSessionArchiveError};

impl RuntimeSessionArchive {
    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), RuntimeSessionArchiveError> {
        io::save_to_path(self, path)
    }
}
