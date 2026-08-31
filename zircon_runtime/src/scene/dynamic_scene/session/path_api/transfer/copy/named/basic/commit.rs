use std::path::Path;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, path_transfer,
};

impl RuntimeSessionArchive {
    pub fn copy_slot_at_path_atomically(
        path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::copy_slot_at_path_atomically(path, source_slot_id, new_slot_id)
    }
}
