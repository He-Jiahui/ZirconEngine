#[path = "../../../../../import/loaded/named/basic/path_transfer_commit.rs"]
mod commit;
#[path = "../../../../../import/loaded/named/basic/path_transfer_preview.rs"]
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::import_slot_from_archive_at_path_atomically;
pub(in crate::scene::dynamic_scene::session) use preview::preview_import_slot_from_archive_at_path;
