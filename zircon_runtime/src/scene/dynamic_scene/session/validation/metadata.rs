use super::super::RuntimeSessionArchive;

pub(in crate::scene::dynamic_scene::session) fn normalize_slot_metadata(
    archive: &mut RuntimeSessionArchive,
) {
    for slot in &mut archive.slots {
        slot.metadata.normalize();
    }
    archive.rebuild_slot_indexes();
}
