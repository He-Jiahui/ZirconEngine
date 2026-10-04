//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "native_plugin_loader/abi_surfaces.rs"]
mod abi_surfaces;
#[path = "native_plugin_loader/bridge_lifecycle.rs"]
mod bridge_lifecycle;
#[path = "native_plugin_loader/diagnostics.rs"]
mod diagnostics;
#[path = "native_plugin_loader/live_host.rs"]
mod live_host;
#[path = "native_plugin_loader/manifest_sources.rs"]
mod manifest_sources;
