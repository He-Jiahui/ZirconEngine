//! 捕获与裁剪共享一份场景快照及代际绑定计划；预检先完成，提交再统一发布目标槽位和删除集合。
mod apply;
mod level;
mod world;

pub(super) use level::{
    capture_level_slot_with_retention, capture_level_slot_with_tag_retention,
    preview_level_slot_with_retention, preview_level_slot_with_tag_retention,
};
pub(super) use world::{
    capture_world_slot_with_retention, capture_world_slot_with_tag_retention,
    preview_world_slot_with_retention, preview_world_slot_with_tag_retention,
};
