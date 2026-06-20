use std::path::Path;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn save_selected_single_slot_archive_to_path_atomically(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();
    let report =
        super::preview_save_selected_single_slot_archive_to_path(archive, selector, target_path)?;
    let exported_archive = archive.single_slot_archive(&report.source_slot_id)?;
    io::save_to_path_atomically(&exported_archive, target_path)?;
    exported_archive.manifest()
}
