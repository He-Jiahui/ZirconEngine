mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{apply_slot, apply_slot_to_level};
pub(in crate::scene::dynamic_scene::session) use selected::{
    apply_selected_slot, apply_selected_slot_to_level,
};
