mod basic;
mod metadata;

pub(in crate::scene::dynamic_scene::session) use basic::{
    copy_selected_slot, preview_copy_selected_slot,
};
pub(in crate::scene::dynamic_scene::session) use metadata::{
    copy_selected_slot_with_metadata, preview_copy_selected_slot_with_metadata,
};
