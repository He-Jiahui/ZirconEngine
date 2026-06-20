use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::metadata::import_slot_from_archive_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let metadata = incoming.require_slot(source_slot_id)?.metadata.clone();
    import_slot_from_archive_with_metadata(target, incoming, source_slot_id, new_slot_id, metadata)
}
