//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "replay_and_runtime/bridge_methods.rs"]
mod bridge_methods;
#[path = "replay_and_runtime/registration_replay.rs"]
mod registration_replay;
#[path = "replay_and_runtime/runtime_behavior.rs"]
mod runtime_behavior;
