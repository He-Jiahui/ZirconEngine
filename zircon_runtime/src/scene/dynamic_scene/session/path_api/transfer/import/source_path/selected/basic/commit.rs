use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn import_selected_slot_from_archive_path_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_selected_slot_from_archive_path_at_path_atomically(
            path,
            source_path,
            selector,
            new_slot_id,
        )
    }
}
