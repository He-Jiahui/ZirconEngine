use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_copy_slot_with_metadata(
        &self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_slot_with_metadata(self, source_slot_id, new_slot_id, metadata)
    }
}
