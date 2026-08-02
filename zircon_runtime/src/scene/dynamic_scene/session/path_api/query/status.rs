use std::path::Path;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchivePathStatus, path_query};

impl RuntimeSessionArchive {
    pub fn inspect_path(path: impl AsRef<Path>) -> RuntimeSessionArchivePathStatus {
        path_query::inspect_path(path)
    }
}
