//! 跨档案单槽位导入的源元数据继承契约，同时向上提供只读预览与内存提交。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::import_selected_slot_from_archive;
pub(in crate::scene::dynamic_scene::session) use preview::preview_import_selected_slot_from_archive;
