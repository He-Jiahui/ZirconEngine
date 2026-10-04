//! 完整存档的路径载入、预览和保存入口。load_or_empty_from_path 只在允许首次创建存档的流程中使用；
//! 普通载入保留“路径缺失”错误。预览描述检查时的目标状态，不为后续保存保留写入权限。

mod atomic;
mod load;
mod preview;
mod save;
