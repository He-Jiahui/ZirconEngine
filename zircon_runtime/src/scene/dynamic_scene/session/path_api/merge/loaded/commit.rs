use std::path::Path;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, path_merge,
};

impl RuntimeSessionArchive {
    pub fn merge_archive_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::merge_archive_at_path_atomically(path, incoming, policy)
    }
}
