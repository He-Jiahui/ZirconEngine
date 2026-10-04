//! 跨档案单槽位导入的内存操作边界；新 ID 必须空闲，源场景保留，持久化由路径传输入口负责。
mod named;
mod selected;

pub(super) use named::{
    import_slot_from_archive, import_slot_from_archive_with_metadata,
    preview_import_slot_from_archive, preview_import_slot_from_archive_with_metadata,
};
pub(super) use selected::{
    import_selected_slot_from_archive, import_selected_slot_from_archive_with_metadata,
    preview_import_selected_slot_from_archive,
    preview_import_selected_slot_from_archive_with_metadata,
};
