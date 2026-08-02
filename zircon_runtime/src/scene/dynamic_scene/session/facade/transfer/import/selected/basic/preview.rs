use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector, slot_import,
};

impl RuntimeSessionArchive {
    pub fn preview_import_selected_slot_from_archive(
        &self,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_import::preview_import_selected_slot_from_archive(
            self,
            incoming,
            selector,
            new_slot_id,
        )
    }
}
