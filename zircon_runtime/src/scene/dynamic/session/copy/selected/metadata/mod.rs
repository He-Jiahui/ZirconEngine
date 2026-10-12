#[path = "../../metadata/path_transfer_commit.rs"]
mod commit;
#[path = "../../metadata/path_transfer_preview.rs"]
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::copy_selected_slot_with_metadata_at_path_atomically;
pub(in crate::scene::dynamic_scene::session) use preview::preview_copy_selected_slot_with_metadata_from_path;
