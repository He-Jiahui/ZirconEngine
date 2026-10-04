//! 已加载源复用调用方持有的存档快照；磁盘源先检查源与目标的路径身份，再载入外部存档。
//! 两种来源最终进入目标路径的同一类导入流程，预览和提交分别计算与发布结果。

mod loaded;
mod source_path;

pub(in crate::scene::dynamic_scene::session) use loaded::{
    import_selected_slot_from_archive_at_path_atomically,
    import_selected_slot_from_archive_with_metadata_at_path_atomically,
    import_slot_from_archive_at_path_atomically,
    import_slot_from_archive_with_metadata_at_path_atomically,
    preview_import_selected_slot_from_archive_at_path,
    preview_import_selected_slot_from_archive_with_metadata_at_path,
    preview_import_slot_from_archive_at_path,
    preview_import_slot_from_archive_with_metadata_at_path,
};
pub(in crate::scene::dynamic_scene::session) use source_path::{
    import_selected_slot_from_archive_path_at_path_atomically,
    import_selected_slot_from_archive_path_with_metadata_at_path_atomically,
    import_slot_from_archive_path_at_path_atomically,
    import_slot_from_archive_path_with_metadata_at_path_atomically,
    preview_import_selected_slot_from_archive_path_at_path,
    preview_import_selected_slot_from_archive_path_with_metadata_at_path,
    preview_import_slot_from_archive_path_at_path,
    preview_import_slot_from_archive_path_with_metadata_at_path,
};
