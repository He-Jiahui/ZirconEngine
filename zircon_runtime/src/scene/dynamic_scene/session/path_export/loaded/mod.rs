mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{
    preview_save_single_slot_archive_to_path, save_single_slot_archive_to_path_atomically,
};
pub(in crate::scene::dynamic_scene::session) use selected::{
    preview_save_selected_single_slot_archive_to_path,
    save_selected_single_slot_archive_to_path_atomically,
};
