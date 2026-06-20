use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_copy_slot;

pub(in crate::scene::dynamic_scene::session) fn preview_copy_selected_slot(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    preview_copy_slot(archive, &report.selected_slot_id, new_slot_id)
}
