use std::path::Path;

use super::super::super::super::{
    path_merge, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
};

impl RuntimeSessionArchive {
    pub fn preview_merge_archive_from_path_at_path(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::preview_merge_archive_from_path_at_path(path, source_path, policy)
    }
}
