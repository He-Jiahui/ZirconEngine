use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector, slot_copy,
};

impl RuntimeSessionArchive {
    pub fn preview_copy_selected_slot_with_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_selected_slot_with_metadata(self, selector, new_slot_id, metadata)
    }
}
