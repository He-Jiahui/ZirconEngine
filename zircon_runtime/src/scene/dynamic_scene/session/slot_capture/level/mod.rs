//! 命名关卡槽位的捕获边界；关卡元数据随场景形成同一份待提交快照。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::capture_level_slot;
pub(in crate::scene::dynamic_scene::session) use preview::preview_level_slot;
