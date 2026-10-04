//! 按合并策略将外部存档并入目标路径；磁盘源版本先拒绝源与目标指向同一存档。
//! 预览报告只对应当时读到的内容；提交会重新读取目标并原子发布合并后的文件。

mod loaded;
mod source_path;

pub(in crate::scene::dynamic_scene::session) use loaded::{
    merge_archive_at_path_atomically, preview_merge_archive_at_path,
};
pub(in crate::scene::dynamic_scene::session) use source_path::{
    merge_archive_from_path_at_path_atomically, preview_merge_archive_from_path_at_path,
};
