use super::super::super::super::super::super::{
    slot_import, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
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
