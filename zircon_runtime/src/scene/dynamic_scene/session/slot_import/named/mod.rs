//! 用显式源 ID 进行跨档案单槽位导入；普通入口继承源元数据，另一入口接受完整替换元数据。
mod basic;
mod metadata;

pub(in crate::scene::dynamic_scene::session) use basic::{
    import_slot_from_archive, preview_import_slot_from_archive,
};
pub(in crate::scene::dynamic_scene::session) use metadata::{
    import_slot_from_archive_with_metadata, preview_import_slot_from_archive_with_metadata,
};
