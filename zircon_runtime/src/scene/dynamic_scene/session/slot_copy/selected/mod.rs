//! 在当前档案解析选择器后复用命名传输；目标新 ID 始终由调用者指定。
mod basic;
mod metadata;

pub(in crate::scene::dynamic_scene::session) use basic::{
    copy_selected_slot, preview_copy_selected_slot,
};
pub(in crate::scene::dynamic_scene::session) use metadata::{
    copy_selected_slot_with_metadata, preview_copy_selected_slot_with_metadata,
};
