use std::path::Path;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, path_merge,
};

impl RuntimeSessionArchive {
    pub fn merge_archive_from_path_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::merge_archive_from_path_at_path_atomically(path, source_path, policy)
    }
}
