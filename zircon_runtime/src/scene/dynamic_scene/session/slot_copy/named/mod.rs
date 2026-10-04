//! 用显式源 ID 进行同档案槽位复制；普通入口继承源元数据，另一入口接受完整替换元数据。
mod basic;
mod metadata;

pub(in crate::scene::dynamic_scene::session) use basic::{copy_slot, preview_copy_slot};
pub(in crate::scene::dynamic_scene::session) use metadata::{
    copy_slot_with_metadata, preview_copy_slot_with_metadata,
};
