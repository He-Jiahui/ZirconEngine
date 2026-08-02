use std::path::Path;

use super::super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector, path_transfer,
};

impl RuntimeSessionArchive {
    pub fn import_selected_slot_from_archive_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_selected_slot_from_archive_at_path_atomically(
            path,
            incoming,
            selector,
            new_slot_id,
        )
    }
}
