//! 普通捕获的内存预检边界；一次捕获同时生成待提交槽位和对外摘要，允许替换同名槽位。
#[path = "level/mod.rs"]
mod level;
#[path = "preview.rs"]
mod preview;
#[path = "world/mod.rs"]
mod world;

pub(in crate::scene::dynamic_scene::session) use level::{capture_level_slot, preview_level_slot};
pub(in crate::scene::dynamic_scene::session) use preview::RuntimeSessionSlotCapturePreview;
pub(in crate::scene::dynamic_scene::session) use world::{capture_world_slot, preview_world_slot};
