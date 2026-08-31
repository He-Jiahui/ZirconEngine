use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, slot_import,
};

impl RuntimeSessionArchive {
    pub fn preview_import_slot_from_archive_with_metadata(
        &self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_import::preview_import_slot_from_archive_with_metadata(
            self,
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
