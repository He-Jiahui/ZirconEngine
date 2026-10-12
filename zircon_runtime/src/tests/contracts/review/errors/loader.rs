//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "loader/abi_surfaces.rs"]
mod abi_surfaces;
#[path = "loader/bridge_lifecycle.rs"]
mod bridge_lifecycle;
#[path = "loader/diagnostics.rs"]
mod diagnostics;
#[path = "loader/live_host.rs"]
mod live_host;
#[path = "loader/manifest_sources.rs"]
mod manifest_sources;
