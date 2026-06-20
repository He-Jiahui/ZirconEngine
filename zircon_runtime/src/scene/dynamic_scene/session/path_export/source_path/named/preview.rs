use std::path::Path;

use super::super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport,
};

pub(in crate::scene::dynamic_scene::session) fn preview_save_single_slot_archive_from_path(
    source_path: impl AsRef<Path>,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        target_path,
        "runtime session single-slot archive export",
    )?;
    super::super::super::preview_save_single_slot_archive_to_path(
        &io::load_from_path(source_path)?,
        slot_id,
        target_path,
    )
}
