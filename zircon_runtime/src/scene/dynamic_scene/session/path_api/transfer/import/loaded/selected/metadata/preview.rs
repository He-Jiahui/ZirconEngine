use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_import_selected_slot_from_archive_with_metadata_at_path(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_selected_slot_from_archive_with_metadata_at_path(
            path,
            incoming,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
