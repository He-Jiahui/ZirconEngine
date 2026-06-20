use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_copy_slot_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn preview_copy_selected_slot_with_metadata(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    preview_copy_slot_with_metadata(archive, &report.selected_slot_id, new_slot_id, metadata)
}
