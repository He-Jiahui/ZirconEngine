use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport,
};

pub(in crate::scene::dynamic_scene::session) fn preview_merge_archive_at_path(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.preview_merge_archive(incoming, policy)
}
