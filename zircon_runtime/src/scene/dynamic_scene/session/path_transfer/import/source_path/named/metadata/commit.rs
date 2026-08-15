use std::path::Path;

use super::super::super::super::super::super::{
    io, target_path as archive_target_path, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata,
};
use super::super::super::super::loaded::import_slot_from_archive_with_metadata_at_path_atomically;

pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive_path_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let source_path = source_path.as_ref();
    archive_target_path::reject_same_archive_paths(
        source_path,
        path,
        "runtime session single-slot archive import",
    )?;
    let incoming = io::load_from_path(source_path)?;
    import_slot_from_archive_with_metadata_at_path_atomically(
        path,
        &incoming,
        source_slot_id,
        new_slot_id,
        metadata,
    )
}
