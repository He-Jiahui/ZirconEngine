use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionMetadata, RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn update_slot_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        slot_id: &str,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            archive.update_slot_metadata(slot_id, metadata)?;
            Ok(())
        })
    }

    pub fn update_selected_slot_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_atomically(path, |archive| {
            archive.update_selected_slot_metadata(selector, metadata)?;
            Ok(())
        })
    }
}
