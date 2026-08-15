use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_copy_selected_slot_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_copy_selected_slot_from_path(path, selector, new_slot_id)
    }
}
