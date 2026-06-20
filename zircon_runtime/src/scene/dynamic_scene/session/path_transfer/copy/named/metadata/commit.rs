use std::path::Path;

use super::super::super::super::super::{
    io, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionMetadata,
};

pub(in crate::scene::dynamic_scene::session) fn copy_slot_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.copy_slot_with_metadata(source_slot_id, new_slot_id, metadata)?;
        Ok(())
    })
}
