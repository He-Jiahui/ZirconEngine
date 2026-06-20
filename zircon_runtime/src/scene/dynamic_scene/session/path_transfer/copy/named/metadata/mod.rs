mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::copy_slot_with_metadata_at_path_atomically;
pub(in crate::scene::dynamic_scene::session) use preview::preview_copy_slot_with_metadata_from_path;
