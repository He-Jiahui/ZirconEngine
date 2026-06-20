use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    pub fn import_slot_from_archive_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_slot_from_archive_with_metadata_at_path_atomically(
            path,
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
