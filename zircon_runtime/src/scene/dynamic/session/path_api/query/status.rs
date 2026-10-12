use std::path::Path;

use super::super::super::{path_query, RuntimeSessionArchive, RuntimeSessionArchivePathStatus};

impl RuntimeSessionArchive {
    pub fn inspect_path(path: impl AsRef<Path>) -> RuntimeSessionArchivePathStatus {
        path_query::inspect_path(path)
    }
}
