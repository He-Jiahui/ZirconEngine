use std::path::Path;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, path_merge,
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
