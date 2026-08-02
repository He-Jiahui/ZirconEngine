use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    slot_import,
};

impl RuntimeSessionArchive {
    pub fn preview_import_slot_from_archive(
        &self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_import::preview_import_slot_from_archive(self, incoming, source_slot_id, new_slot_id)
    }
}
