//! 将关卡捕获接入全档案或标签裁剪，复用同一份快照与目标保护规则。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::{
    capture_level_slot_with_retention, capture_level_slot_with_tag_retention,
};
pub(in crate::scene::dynamic_scene::session) use preview::{
    preview_level_slot_with_retention, preview_level_slot_with_tag_retention,
};
