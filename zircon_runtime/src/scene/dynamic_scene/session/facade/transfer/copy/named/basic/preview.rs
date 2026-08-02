use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    slot_copy,
};

impl RuntimeSessionArchive {
    pub fn preview_copy_slot(
        &self,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_copy::preview_copy_slot(self, source_slot_id, new_slot_id)
    }
}
