use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotMutationPreviewReport, RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn preview_update_slot_metadata_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_update_slot_metadata(slot_id, metadata)
    }

    pub fn preview_update_selected_slot_metadata_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_update_selected_slot_metadata(selector, metadata)
    }
}
