use super::super::super::{RuntimeSessionArchive, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn remove_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
) -> Option<RuntimeSessionSlot> {
    let index = archive
        .slots
        .iter()
        .position(|slot| slot.slot_id == slot_id)?;
    Some(archive.slots.remove(index))
}
