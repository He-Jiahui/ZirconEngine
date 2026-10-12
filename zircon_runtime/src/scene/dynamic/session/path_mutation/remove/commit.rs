use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn remove_slot_at_path_atomically(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            let report = archive.preview_remove_slot(slot_id)?;
            let removed = archive.remove_slot(&report.source_slot_id);
            debug_assert!(removed.is_some());
            Ok(())
        })
    }

    pub fn remove_selected_slot_at_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            let removed = archive.remove_selected_slot(selector)?;
            drop(removed);
            Ok(())
        })
    }
}
