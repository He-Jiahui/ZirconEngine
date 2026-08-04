use super::super::super::{RuntimeSessionArchive, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn remove_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
) -> Option<RuntimeSessionSlot> {
    let index = archive.indexed_slot_index(slot_id)?;
    let removed = archive.slots.remove(index);
    archive.rebuild_slot_indexes();
    Some(removed)
}
