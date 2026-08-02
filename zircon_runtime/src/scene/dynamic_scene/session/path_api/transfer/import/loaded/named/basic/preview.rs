use std::path::Path;

use super::super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    path_transfer,
};

impl RuntimeSessionArchive {
    pub fn preview_import_slot_from_archive_at_path(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_slot_from_archive_at_path(
            path,
            incoming,
            source_slot_id,
            new_slot_id,
        )
    }
}
