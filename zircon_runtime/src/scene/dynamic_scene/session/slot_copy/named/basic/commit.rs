use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::metadata::copy_slot_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn copy_slot(
    archive: &mut RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let metadata = archive.require_slot(source_slot_id)?.metadata.clone();
    copy_slot_with_metadata(archive, source_slot_id, new_slot_id, metadata)
}
