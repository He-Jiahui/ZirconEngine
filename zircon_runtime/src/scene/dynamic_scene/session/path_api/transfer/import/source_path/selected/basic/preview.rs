use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_import_selected_slot_from_archive_path_at_path(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_selected_slot_from_archive_path_at_path(
            path,
            source_path,
            selector,
            new_slot_id,
        )
    }
}
