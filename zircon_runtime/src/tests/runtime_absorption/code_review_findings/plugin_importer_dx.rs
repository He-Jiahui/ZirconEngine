//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "plugin_importer_dx/d10_bridge_call.rs"]
mod d10_bridge_call;
#[path = "plugin_importer_dx/d11_test_runtime_fixture.rs"]
mod d11_test_runtime_fixture;
#[path = "plugin_importer_dx/d12_runtime_exports.rs"]
mod d12_runtime_exports;
#[path = "plugin_importer_dx/d13_importer_sdk.rs"]
mod d13_importer_sdk;
#[path = "plugin_importer_dx/d1_capability_single_source.rs"]
mod d1_capability_single_source;
#[path = "plugin_importer_dx/d5_editor_authoring.rs"]
mod d5_editor_authoring;
#[path = "plugin_importer_dx/d6_runtime_plugin_id.rs"]
mod d6_runtime_plugin_id;
#[path = "plugin_importer_dx/d8_registration_builder.rs"]
mod d8_registration_builder;
#[path = "plugin_importer_dx/d9_editor_runtime_mirror.rs"]
mod d9_editor_runtime_mirror;
