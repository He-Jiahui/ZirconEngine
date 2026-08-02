use std::path::Path;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, path_export,
};

impl RuntimeSessionArchive {
    pub fn save_single_slot_archive_to_path_atomically(
        &self,
        slot_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_export::save_single_slot_archive_to_path_atomically(self, slot_id, target_path)
    }
}
