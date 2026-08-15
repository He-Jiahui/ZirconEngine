use std::path::Path;

use super::super::super::super::super::{
    path_export, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn selected_single_slot_archive_from_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        path_export::selected_single_slot_archive_from_path(path, selector)
    }

    pub fn save_selected_single_slot_archive_from_path_atomically(
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_export::save_selected_single_slot_archive_from_path_atomically(
            source_path,
            selector,
            target_path,
        )
    }
}
