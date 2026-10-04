//! 生成独立的单槽位档案供导出，源档案保持原样；路径预览只观测目标状态。
mod commit;
mod preview;

pub(in crate::scene::dynamic_scene::session) use commit::{
    selected_single_slot_archive, single_slot_archive,
};
pub(in crate::scene::dynamic_scene::session) use preview::{
    preview_selected_single_slot_archive, preview_single_slot_archive,
    preview_single_slot_archive_to_path,
};
