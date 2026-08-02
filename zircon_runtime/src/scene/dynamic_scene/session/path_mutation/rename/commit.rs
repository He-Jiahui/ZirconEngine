use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn rename_slot_at_path_atomically(
        path: impl AsRef<Path>,
        old_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            archive.rename_slot(old_slot_id, new_slot_id)?;
            Ok(())
        })
    }

    pub fn rename_selected_slot_at_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            archive.rename_selected_slot(selector, new_slot_id)?;
            Ok(())
        })
    }
}
