use std::path::Path;

use super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport,
};

pub(in crate::scene::dynamic_scene::session) fn preview_save_single_slot_archive_to_path(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    slot_export::preview_single_slot_archive_to_path(archive, slot_id, target_path)
}
