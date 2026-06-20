mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{
    diff_slot_with_level, diff_slot_with_world,
};
pub(in crate::scene::dynamic_scene::session) use selected::{
    diff_selected_slot_with_level, diff_selected_slot_with_world,
};
