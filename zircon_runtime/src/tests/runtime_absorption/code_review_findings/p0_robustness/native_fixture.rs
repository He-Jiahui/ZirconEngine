//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "native_fixture/importer_manifest.rs"]
mod importer_manifest;
#[path = "native_fixture/sdk_macro_manifest.rs"]
mod sdk_macro_manifest;
