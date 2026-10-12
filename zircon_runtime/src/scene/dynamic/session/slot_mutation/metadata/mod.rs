//! 整体元数据替换的预览与提交；索引和档案代际由底层写入边界维护。
#[path = "commit.rs"]
mod commit;
#[path = "preview.rs"]
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::update_slot_metadata;
pub(in crate::scene::dynamic_scene::session) use preview::preview_update_slot_metadata;
