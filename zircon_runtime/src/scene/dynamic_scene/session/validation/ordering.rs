use super::super::RuntimeSessionArchive;

pub(in crate::scene::dynamic_scene::session) fn sort_slots(archive: &mut RuntimeSessionArchive) {
    archive
        .slots
        .sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
    archive.rebuild_slot_indexes();
}
