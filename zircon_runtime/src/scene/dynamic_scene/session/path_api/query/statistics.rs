use std::path::Path;

use super::super::super::{
    path_query, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveStatistics,
};

impl RuntimeSessionArchive {
    pub fn statistics_from_path(
        path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
        path_query::statistics_from_path(path)
    }
}
