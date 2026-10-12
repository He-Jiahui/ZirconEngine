use std::path::Path;

use super::super::super::super::{
    slot_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn preview_save_selected_single_slot_archive_to_path(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    slot_export::preview_single_slot_archive_to_path(archive, &report.selected_slot_id, target_path)
}
