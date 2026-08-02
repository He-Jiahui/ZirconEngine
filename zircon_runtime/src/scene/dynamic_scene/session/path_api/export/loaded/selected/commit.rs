use std::path::Path;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector, path_export,
};

impl RuntimeSessionArchive {
    pub fn save_selected_single_slot_archive_to_path_atomically(
        &self,
        selector: RuntimeSessionSlotSelector,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_export::save_selected_single_slot_archive_to_path_atomically(
            self,
            selector,
            target_path,
        )
    }
}
