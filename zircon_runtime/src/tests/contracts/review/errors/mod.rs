//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "animation_resource.rs"]
mod animation_resource;
#[path = "asset_loaders.rs"]
mod asset_loaders;
#[path = "asset_records.rs"]
mod asset_records;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "dynamic_api.rs"]
mod dynamic_api;
#[path = "export_cli.rs"]
mod export_cli;
#[path = "loader.rs"]
mod native_plugin_loader;
#[path = "scene_world.rs"]
mod scene_world;
#[path = "script_host.rs"]
mod script_host;
#[path = "shader_prewarm_cli.rs"]
mod shader_prewarm_cli;
#[path = "ui_asset_documents.rs"]
mod ui_asset_documents;
#[path = "ui_input.rs"]
mod ui_input;
#[path = "ui_template_resource.rs"]
mod ui_template_resource;
