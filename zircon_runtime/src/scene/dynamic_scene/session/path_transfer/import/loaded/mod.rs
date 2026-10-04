//! 已加载来源的路径执行入口；目标档案必须存在，预览结果不锁定后续提交。
mod named;
mod selected;

pub(in crate::scene::dynamic_scene::session) use named::{
    import_slot_from_archive_at_path_atomically,
    import_slot_from_archive_with_metadata_at_path_atomically,
    preview_import_slot_from_archive_at_path,
    preview_import_slot_from_archive_with_metadata_at_path,
};
pub(in crate::scene::dynamic_scene::session) use selected::{
    import_selected_slot_from_archive_at_path_atomically,
    import_selected_slot_from_archive_with_metadata_at_path_atomically,
    preview_import_selected_slot_from_archive_at_path,
    preview_import_selected_slot_from_archive_with_metadata_at_path,
};
