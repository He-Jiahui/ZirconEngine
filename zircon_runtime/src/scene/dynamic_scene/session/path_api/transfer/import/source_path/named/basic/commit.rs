use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};

impl RuntimeSessionArchive {
    pub fn import_slot_from_archive_path_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_slot_from_archive_path_at_path_atomically(
            path,
            source_path,
            source_slot_id,
            new_slot_id,
        )
    }
}
