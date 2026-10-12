//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。保存同组守卫使用的局部数据或辅助变换。
#[path = "../runtime_15_m2/asset_dynamic.rs"]
mod asset_dynamic;
#[path = "runtime_15_m2/asset_schema.rs"]
mod asset_schema;
#[path = "runtime_15_m2/banned_names.rs"]
mod banned_names;
#[path = "../runtime_15_m2/core_framework.rs"]
mod core_framework;
#[path = "runtime_15_m2/core_scene.rs"]
mod core_scene;
#[path = "../runtime_15_m2/editor_workbench.rs"]
mod editor_workbench;
#[path = "runtime_15_m2/graphics.rs"]
mod graphics;
#[path = "runtime_15_m2/hub.rs"]
mod hub;
#[path = "runtime_15_m2/input.rs"]
mod input;
#[path = "runtime_15_m2/net.rs"]
mod net;
#[path = "../runtime_15_m2/plugin_static_manifest.rs"]
mod plugin_static_manifest;
#[path = "runtime_15_m2/scene.rs"]
mod scene_tests;
#[path = "runtime_15_m2/ui.rs"]
mod ui;
