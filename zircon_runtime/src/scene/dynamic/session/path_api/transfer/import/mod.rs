//! 路径单槽导入入口：已加载来源使用调用者快照，磁盘来源由独立入口加载。
#[path = "loaded/mod.rs"]
mod loaded;
#[path = "../../../transfer/import/source_path/mod.rs"]
mod source_path;
