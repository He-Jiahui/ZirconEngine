use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn preview_rename_slot_from_path(
        path: impl AsRef<Path>,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_rename_slot(old_slot_id, new_slot_id)
    }

    pub fn preview_rename_selected_slot_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_rename_selected_slot(selector, new_slot_id)
    }
}
