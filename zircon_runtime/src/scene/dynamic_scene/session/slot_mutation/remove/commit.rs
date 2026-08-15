use super::super::super::{RuntimeSessionArchive, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn remove_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
) -> Option<RuntimeSessionSlot> {
    archive.remove_indexed_slot(slot_id)
}
