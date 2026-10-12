use std::path::Path;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionMetadata,
};

pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.import_slot_from_archive_with_metadata(
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )?;
        Ok(())
    })
}
