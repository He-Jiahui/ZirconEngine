use super::{assert_contains_all, repo_path, runtime_src_path};

#[path = "production_file_budget/core_runtime_service_lists.rs"]
mod core_runtime_service_lists;
#[path = "production_file_budget/hzb_occlusion_culler.rs"]
mod hzb_occlusion_culler;
#[path = "production_file_budget/material_asset.rs"]
mod material_asset;
#[path = "production_file_budget/module_layout.rs"]
mod module_layout;
#[path = "production_file_budget/render_backend_types.rs"]
mod render_backend_types;
#[path = "production_file_budget/render_build_virtual_geometry_debug_snapshot.rs"]
mod render_build_virtual_geometry_debug_snapshot;
#[path = "production_file_budget/render_scene_world.rs"]
mod render_scene_world;
#[path = "production_file_budget/render_shadow.rs"]
mod render_shadow;
#[path = "production_file_budget/render_stats_graph.rs"]
mod render_stats_graph;
#[path = "production_file_budget/render_submit_camera_loop.rs"]
mod render_submit_camera_loop;
#[path = "production_file_budget/render_ui_screen_space_render.rs"]
mod render_ui_screen_space_render;
#[path = "production_file_budget/render_update_base_stats.rs"]
mod render_update_base_stats;
#[path = "production_file_budget/render_vg_debug_snapshot_streams.rs"]
mod render_vg_debug_snapshot_streams;
#[path = "production_file_budget/rhi_wgpu_command_validation.rs"]
mod rhi_wgpu_command_validation;
#[path = "production_file_budget/rhi_wgpu_ui_surface_geometry.rs"]
mod rhi_wgpu_ui_surface_geometry;
#[path = "production_file_budget/rhi_wgpu_ui_surface_render_setup.rs"]
mod rhi_wgpu_ui_surface_render_setup;
#[path = "production_file_budget/scene_fixed_lights.rs"]
mod scene_fixed_lights;
#[path = "production_file_budget/scene_world_project_io.rs"]
mod scene_world_project_io;
#[path = "production_file_budget/scene_world_property_access.rs"]
mod scene_world_property_access;
#[path = "production_file_budget/ui_accessibility_extract.rs"]
mod ui_accessibility_extract;
#[path = "production_file_budget/ui_component_catalog_editor_showcase.rs"]
mod ui_component_catalog_editor_showcase;
#[path = "production_file_budget/ui_component_state_reducer_keyboard_menu.rs"]
mod ui_component_state_reducer_keyboard_menu;
#[path = "production_file_budget/ui_component_state_reducer_tree_view.rs"]
mod ui_component_state_reducer_tree_view;
#[path = "production_file_budget/ui_layout_arrange.rs"]
mod ui_layout_arrange;
#[path = "production_file_budget/ui_surface_default_interactions.rs"]
mod ui_surface_default_interactions;
#[path = "production_file_budget/ui_surface_event_routing.rs"]
mod ui_surface_event_routing;
#[path = "production_file_budget/ui_surface_property_mutation.rs"]
mod ui_surface_property_mutation;
#[path = "production_file_budget/ui_surface_render_feedback.rs"]
mod ui_surface_render_feedback;
#[path = "production_file_budget/ui_surface_table_columns.rs"]
mod ui_surface_table_columns;
#[path = "production_file_budget/ui_template_document.rs"]
mod ui_template_document;
#[path = "production_file_budget/ui_template_style_apply.rs"]
mod ui_template_style_apply;
#[path = "production_file_budget/ui_text_layout.rs"]
mod ui_text_layout;
#[path = "production_file_budget/ui_v2_style.rs"]
mod ui_v2_style;

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
