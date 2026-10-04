//! 从磁盘槽位比较、应用或恢复场景：diff 不改目标，apply 写入现有场景，restore 可创建空 World 或替换关卡。
//! 调用方须按目标语义选择入口，并处理 apply 返回的实体映射或关卡恢复报告。

mod apply;
mod diff;
mod restore;

pub(super) use apply::{
    apply_selected_slot_from_path_to_level, apply_selected_slot_from_path_to_world,
    apply_slot_from_path_to_level, apply_slot_from_path_to_world,
};
pub(super) use diff::{
    diff_selected_slot_from_path_with_level, diff_selected_slot_from_path_with_world,
    diff_slot_from_path_with_level, diff_slot_from_path_with_world,
};
pub(super) use restore::{
    restore_selected_slot_from_path_into_level, restore_selected_slot_from_path_to_empty_world,
    restore_slot_from_path_into_level, restore_slot_from_path_to_empty_world,
};
