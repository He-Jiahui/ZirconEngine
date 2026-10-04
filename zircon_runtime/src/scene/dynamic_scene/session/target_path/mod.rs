//! 保存、导出、导入与合并共用的路径预检；检查当前文件状态，不建立文件锁或发布权。
mod source;
mod target;

pub(in crate::scene::dynamic_scene::session) use source::reject_same_archive_paths;
pub(in crate::scene::dynamic_scene::session) use target::target_file_will_replace;
