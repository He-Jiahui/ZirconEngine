//! 同档案槽位复制的源元数据继承契约，同时向上提供只读预览与内存提交。
#[path = "commit.rs"]
mod commit;
#[path = "preview.rs"]
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::copy_selected_slot;
pub(in crate::scene::dynamic_scene::session) use preview::preview_copy_selected_slot;
