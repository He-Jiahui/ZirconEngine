use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn require_slot<'a>(
    archive: &'a RuntimeSessionArchive,
    slot_id: &str,
) -> Result<&'a RuntimeSessionSlot, RuntimeSessionArchiveError> {
    archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })
}
