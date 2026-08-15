use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_copy_slot_with_metadata_from_path(
        path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_copy_slot_with_metadata_from_path(
            path,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
