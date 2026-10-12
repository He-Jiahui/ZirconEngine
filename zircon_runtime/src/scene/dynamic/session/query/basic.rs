use super::super::{RuntimeSessionArchive, RuntimeSessionSlot};

pub(in crate::scene::dynamic_scene::session) fn slot_count(
    archive: &RuntimeSessionArchive,
) -> usize {
    archive.payload_arc().slot_count()
}

pub(in crate::scene::dynamic_scene::session) fn is_empty(archive: &RuntimeSessionArchive) -> bool {
    archive.payload_arc().is_empty()
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
    archive.indexed_slot(slot_id)
}

pub(in crate::scene::dynamic_scene::session) fn slots(
    archive: &RuntimeSessionArchive,
) -> impl Iterator<Item = &RuntimeSessionSlot> {
    archive.iter_canonical_slots()
}

pub(in crate::scene::dynamic_scene::session) fn slot_ids(
    archive: &RuntimeSessionArchive,
) -> impl Iterator<Item = &str> {
    archive
        .iter_canonical_slots()
        .map(|slot| slot.slot_id.as_str())
}
