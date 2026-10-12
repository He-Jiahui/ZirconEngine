//! 在源档案解析选择器后复用命名传输；目标新 ID 始终由调用者指定。
#[path = "basic/mod.rs"]
mod basic;
#[path = "metadata/mod.rs"]
mod metadata;

pub(in crate::scene::dynamic_scene::session) use basic::{
    import_selected_slot_from_archive, preview_import_selected_slot_from_archive,
};
pub(in crate::scene::dynamic_scene::session) use metadata::{
    import_selected_slot_from_archive_with_metadata,
    preview_import_selected_slot_from_archive_with_metadata,
};
