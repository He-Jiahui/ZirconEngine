mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{
    copy_slot_at_path_atomically, copy_slot_with_metadata_at_path_atomically,
    preview_copy_slot_from_path, preview_copy_slot_with_metadata_from_path,
};
pub(in crate::scene::dynamic_scene::session) use selected::{
    copy_selected_slot_at_path_atomically, copy_selected_slot_with_metadata_at_path_atomically,
    preview_copy_selected_slot_from_path, preview_copy_selected_slot_with_metadata_from_path,
};
