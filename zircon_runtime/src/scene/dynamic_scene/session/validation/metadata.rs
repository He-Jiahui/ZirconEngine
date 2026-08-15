use super::super::RuntimeSessionArchive;

pub(in crate::scene::dynamic_scene::session) fn normalize_slot_metadata(
    archive: &mut RuntimeSessionArchive,
) {
    archive.normalize_slot_metadata_rows();
}
