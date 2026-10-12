use std::path::Path;

use super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
};
use super::super::loaded::merge_archive_at_path_atomically;

pub(in crate::scene::dynamic_scene::session) fn merge_archive_from_path_at_path_atomically(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let source_path = source_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        path,
        "runtime session archive merge",
    )?;
    let incoming = io::load_from_path(source_path)?;
    merge_archive_at_path_atomically(path, &incoming, policy)
}
