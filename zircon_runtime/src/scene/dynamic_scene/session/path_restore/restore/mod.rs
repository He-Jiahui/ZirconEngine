mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{
    restore_slot_from_path_into_level, restore_slot_from_path_to_empty_world,
};
pub(in crate::scene::dynamic_scene::session) use selected::{
    restore_selected_slot_from_path_into_level, restore_selected_slot_from_path_to_empty_world,
};
