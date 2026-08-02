use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, io,
};

pub(in crate::scene::dynamic_scene::session) fn merge_archive_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_with_report_atomically(path, |archive| {
        archive.merge_archive(incoming, policy)
    })
}
