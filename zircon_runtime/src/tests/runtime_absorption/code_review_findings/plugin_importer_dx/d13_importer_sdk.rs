//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "d13_importer_sdk/manifest_parity.rs"]
mod manifest_parity;
#[path = "d13_importer_sdk/runtime_crates.rs"]
mod runtime_crates;
#[path = "d13_importer_sdk/runtime_exports.rs"]
mod runtime_exports;
#[path = "d13_importer_sdk/runtime_manifests.rs"]
mod runtime_manifests;
