use std::path::Path;

use super::super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector, path_transfer,
};

impl RuntimeSessionArchive {
    pub fn preview_import_selected_slot_from_archive_at_path(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_selected_slot_from_archive_at_path(
            path,
            incoming,
            selector,
            new_slot_id,
        )
    }
}
