use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn import_selected_slot_from_archive_path_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_selected_slot_from_archive_path_with_metadata_at_path_atomically(
            path,
            source_path,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
