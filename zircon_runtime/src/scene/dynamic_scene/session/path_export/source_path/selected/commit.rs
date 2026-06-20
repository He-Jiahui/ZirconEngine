use std::path::Path;

use super::super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn selected_single_slot_archive_from_path(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.selected_single_slot_archive(selector)
}

pub(in crate::scene::dynamic_scene::session) fn save_selected_single_slot_archive_from_path_atomically(
    source_path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        target_path,
        "runtime session selected single-slot archive export",
    )?;

    let source_archive = io::load_from_path(source_path)?;
    super::super::super::save_selected_single_slot_archive_to_path_atomically(
        &source_archive,
        selector,
        target_path,
    )
}
