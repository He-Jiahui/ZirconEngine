//! 跨档案单槽位导入的元数据替换契约，同时向上提供只读预览与内存提交。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::import_slot_from_archive_with_metadata;
pub(in crate::scene::dynamic_scene::session) use preview::preview_import_slot_from_archive_with_metadata;
