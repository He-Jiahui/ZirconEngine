use super::super::super::super::super::super::{
    slot_import, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_import_selected_slot_from_archive_with_metadata(
        &self,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_import::preview_import_selected_slot_from_archive_with_metadata(
            self,
            incoming,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
