use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn touch_slot_at_path_atomically(
        path: impl AsRef<Path>,
        slot_id: &str,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            archive.touch_slot(slot_id, updated_at_unix_millis)?;
            Ok(())
        })
    }

    pub fn touch_selected_slot_at_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        updated_at_unix_millis: u64,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            archive.touch_selected_slot(selector, updated_at_unix_millis)?;
            Ok(())
        })
    }
}
