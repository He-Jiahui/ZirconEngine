mod basic;
mod metadata;

pub(in crate::scene::dynamic_scene::session) use basic::{
    copy_selected_slot_at_path_atomically, preview_copy_selected_slot_from_path,
};
pub(in crate::scene::dynamic_scene::session) use metadata::{
    copy_selected_slot_with_metadata_at_path_atomically,
    preview_copy_selected_slot_with_metadata_from_path,
};
