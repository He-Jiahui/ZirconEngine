use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_touch_slot_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_touch_slot(slot_id, updated_at_unix_millis)
    }

    pub fn preview_touch_selected_slot_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_touch_selected_slot(selector, updated_at_unix_millis)
    }
}
