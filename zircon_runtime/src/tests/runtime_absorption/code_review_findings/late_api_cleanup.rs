//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "late_api_cleanup/f11_shading_model_registry.rs"]
mod f11_shading_model_registry;
#[path = "late_api_cleanup/f15_editor_pane_data_conversion.rs"]
mod f15_editor_pane_data_conversion;
#[path = "late_api_cleanup/f17_entity_path_lookup.rs"]
mod f17_entity_path_lookup;
#[path = "late_api_cleanup/f18_asset_manager_resolution.rs"]
mod f18_asset_manager_resolution;
#[path = "late_api_cleanup/f19_scene_renderer_construction.rs"]
mod f19_scene_renderer_construction;
