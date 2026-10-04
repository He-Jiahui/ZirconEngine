//! 槽位改名的预览与提交；索引和档案代际由底层写入边界维护。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::rename_slot;
pub(in crate::scene::dynamic_scene::session) use preview::preview_rename_slot;
