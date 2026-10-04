//! 从已加载存档或磁盘源抽取一个槽位，生成可单独保存的存档，供单槽位导出流程使用。
//! 从磁盘源另存到目标路径时先拒绝同一存档，避免导出结果覆盖来源。

mod loaded;
mod source_path;

pub(super) use loaded::{
    preview_save_selected_single_slot_archive_to_path, preview_save_single_slot_archive_to_path,
    save_selected_single_slot_archive_to_path_atomically,
    save_single_slot_archive_to_path_atomically,
};
pub(super) use source_path::{
    preview_save_selected_single_slot_archive_from_path,
    preview_save_single_slot_archive_from_path,
    save_selected_single_slot_archive_from_path_atomically,
    save_single_slot_archive_from_path_atomically, selected_single_slot_archive_from_path,
    single_slot_archive_from_path,
};
