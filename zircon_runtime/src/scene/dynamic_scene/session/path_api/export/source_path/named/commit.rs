use std::path::Path;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, path_export,
};

impl RuntimeSessionArchive {
    pub fn single_slot_archive_from_path(
        path: impl AsRef<Path>,
        slot_id: &str,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        path_export::single_slot_archive_from_path(path, slot_id)
    }

    pub fn save_single_slot_archive_from_path_atomically(
        source_path: impl AsRef<Path>,
        slot_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_export::save_single_slot_archive_from_path_atomically(
            source_path,
            slot_id,
            target_path,
        )
    }
}
