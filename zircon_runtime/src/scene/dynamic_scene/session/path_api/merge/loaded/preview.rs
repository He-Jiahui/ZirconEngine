use std::path::Path;

use super::super::super::super::{
    path_merge, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
};

impl RuntimeSessionArchive {
    pub fn preview_merge_archive_at_path(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::preview_merge_archive_at_path(path, incoming, policy)
    }
}
