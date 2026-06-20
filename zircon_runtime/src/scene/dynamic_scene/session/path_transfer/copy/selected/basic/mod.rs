mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::copy_selected_slot_at_path_atomically;
pub(in crate::scene::dynamic_scene::session) use preview::preview_copy_selected_slot_from_path;
