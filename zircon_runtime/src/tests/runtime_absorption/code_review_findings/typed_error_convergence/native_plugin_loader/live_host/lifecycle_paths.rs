//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "lifecycle_paths/hot_reload.rs"]
mod hot_reload;
#[path = "lifecycle_paths/lifecycle.rs"]
mod lifecycle;
#[path = "lifecycle_paths/loading.rs"]
mod loading;
