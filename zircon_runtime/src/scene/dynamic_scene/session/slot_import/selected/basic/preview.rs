use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_import_slot_from_archive;

pub(in crate::scene::dynamic_scene::session) fn preview_import_selected_slot_from_archive(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    preview_import_slot_from_archive(target, incoming, &report.selected_slot_id, new_slot_id)
}
