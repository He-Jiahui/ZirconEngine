//! 槽位到 World/Level 的消费入口：diff 仅读取，apply 合入现有世界，restore 从空世界替换。

mod apply;
mod diff;
mod restore;

pub(super) use apply::{
    apply_selected_slot, apply_selected_slot_to_level, apply_slot, apply_slot_to_level,
};
pub(super) use diff::{
    diff_selected_slot_with_level, diff_selected_slot_with_world, diff_slot_with_level,
    diff_slot_with_world,
};
pub(super) use restore::{
    restore_selected_slot_into_level, restore_selected_slot_to_empty_world,
    restore_slot_into_level, restore_slot_to_empty_world,
};
