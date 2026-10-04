//! 同档案槽位复制的元数据替换契约，同时向上提供只读预览与内存提交。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::copy_slot_with_metadata;
pub(in crate::scene::dynamic_scene::session) use preview::preview_copy_slot_with_metadata;
