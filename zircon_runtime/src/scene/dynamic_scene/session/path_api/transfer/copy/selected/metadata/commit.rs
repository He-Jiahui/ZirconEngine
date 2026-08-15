use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn copy_selected_slot_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::copy_selected_slot_with_metadata_at_path_atomically(
            path,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
