//! 命名世界槽位的捕获边界；调用者元数据作为完整替换值进入快照。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::capture_world_slot;
pub(in crate::scene::dynamic_scene::session) use preview::preview_world_slot;
