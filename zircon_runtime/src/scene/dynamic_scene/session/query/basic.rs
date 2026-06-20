use super::super::{RuntimeSessionArchive, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn slot_count(
    archive: &RuntimeSessionArchive,
) -> usize {
    archive.slots.len()
}

pub(in crate::scene::dynamic_scene::session) fn is_empty(archive: &RuntimeSessionArchive) -> bool {
    archive.slots.is_empty()
}

pub(in crate::scene::dynamic_scene::session) fn contains_slot(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> bool {
    slot(archive, slot_id).is_some()
}

pub(in crate::scene::dynamic_scene::session) fn slot<'a>(
    archive: &'a RuntimeSessionArchive,
    slot_id: &str,
) -> Option<&'a RuntimeSessionSlot> {
    archive.slots.iter().find(|slot| slot.slot_id == slot_id)
}

pub(in crate::scene::dynamic_scene::session) fn slots(
    archive: &RuntimeSessionArchive,
) -> &[RuntimeSessionSlot] {
    &archive.slots
}

pub(in crate::scene::dynamic_scene::session) fn slot_ids(
    archive: &RuntimeSessionArchive,
) -> impl Iterator<Item = &str> {
    archive.slots.iter().map(|slot| slot.slot_id.as_str())
}
