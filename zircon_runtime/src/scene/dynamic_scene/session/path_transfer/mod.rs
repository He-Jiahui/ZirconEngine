//! 在同一存档中复制槽位，或从另一个已加载／磁盘存档导入槽位。
//! 显式 ID、选择器和元数据继承分别成组，调用方应先确定来源与目标的槽位身份。

mod copy;
mod import;

pub(super) use copy::{
    copy_selected_slot_at_path_atomically, copy_selected_slot_with_metadata_at_path_atomically,
    copy_slot_at_path_atomically, copy_slot_with_metadata_at_path_atomically,
    preview_copy_selected_slot_from_path, preview_copy_selected_slot_with_metadata_from_path,
    preview_copy_slot_from_path, preview_copy_slot_with_metadata_from_path,
};
pub(super) use import::{
    import_selected_slot_from_archive_at_path_atomically,
    import_selected_slot_from_archive_path_at_path_atomically,
    import_selected_slot_from_archive_path_with_metadata_at_path_atomically,
    import_selected_slot_from_archive_with_metadata_at_path_atomically,
    import_slot_from_archive_at_path_atomically, import_slot_from_archive_path_at_path_atomically,
    import_slot_from_archive_path_with_metadata_at_path_atomically,
    import_slot_from_archive_with_metadata_at_path_atomically,
    preview_import_selected_slot_from_archive_at_path,
    preview_import_selected_slot_from_archive_path_at_path,
    preview_import_selected_slot_from_archive_path_with_metadata_at_path,
    preview_import_selected_slot_from_archive_with_metadata_at_path,
    preview_import_slot_from_archive_at_path, preview_import_slot_from_archive_path_at_path,
    preview_import_slot_from_archive_path_with_metadata_at_path,
    preview_import_slot_from_archive_with_metadata_at_path,
};
