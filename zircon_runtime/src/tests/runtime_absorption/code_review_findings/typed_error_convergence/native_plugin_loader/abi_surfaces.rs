//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "abi_surfaces/behavior_bridge.rs"]
mod behavior_bridge;
#[path = "abi_surfaces/host_adapter.rs"]
mod host_adapter;
#[path = "abi_surfaces/plugin_descriptor.rs"]
mod plugin_descriptor;
