//! 将世界捕获及显式元数据接入裁剪；预览报告和提交均来自同一计划构造边界。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::{
    capture_world_slot_with_retention, capture_world_slot_with_tag_retention,
};
pub(in crate::scene::dynamic_scene::session) use preview::{
    preview_world_slot_with_retention, preview_world_slot_with_tag_retention,
};
