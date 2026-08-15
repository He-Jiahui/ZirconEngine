use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_import_slot_from_archive_path_with_metadata_at_path(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_slot_from_archive_path_with_metadata_at_path(
            path,
            source_path,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
