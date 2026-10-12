use std::path::Path;

use super::super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest,
};

pub(in crate::scene::dynamic_scene::session) fn single_slot_archive_from_path(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.single_slot_archive(slot_id)
}

pub(in crate::scene::dynamic_scene::session) fn save_single_slot_archive_from_path_atomically(
    source_path: impl AsRef<Path>,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        target_path,
        "runtime session single-slot archive export",
    )?;

    let source_archive = io::load_from_path(source_path)?;
    super::super::super::save_single_slot_archive_to_path_atomically(
        &source_archive,
        slot_id,
        target_path,
    )
}
