mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{
    apply_slot_from_path_to_level, apply_slot_from_path_to_world,
};
pub(in crate::scene::dynamic_scene::session) use selected::{
    apply_selected_slot_from_path_to_level, apply_selected_slot_from_path_to_world,
};
