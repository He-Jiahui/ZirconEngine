//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "../../../../review/errors/loader/live_host/lifecycle_paths.rs"]
mod lifecycle_paths;
#[path = "../../../../review/errors/loader/live_host/replay_and_runtime.rs"]
mod replay_and_runtime;
