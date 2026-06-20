use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_remove_slot_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_remove_slot(slot_id)
    }

    pub fn preview_remove_selected_slot_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_remove_selected_slot(selector)
    }
}
