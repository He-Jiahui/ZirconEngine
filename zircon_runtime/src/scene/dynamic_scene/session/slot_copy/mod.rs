//! 同档案槽位复制的内存操作边界；新 ID 必须空闲，源场景保留，持久化由路径传输入口负责。
mod named;
mod selected;

pub(super) use named::{
    copy_slot, copy_slot_with_metadata, preview_copy_slot, preview_copy_slot_with_metadata,
};
pub(super) use selected::{
    copy_selected_slot, copy_selected_slot_with_metadata, preview_copy_selected_slot,
    preview_copy_selected_slot_with_metadata,
};
