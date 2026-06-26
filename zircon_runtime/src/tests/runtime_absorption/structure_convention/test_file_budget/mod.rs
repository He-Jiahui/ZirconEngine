use super::{assert_contains_all, repo_path, runtime_src_path};

mod asset_artifact_store;
mod asset_gltf_importer;
mod asset_gltf_primitive_fixtures;
mod asset_importer;
mod asset_mesh;
mod asset_pipeline_manager;
mod asset_project_example_vampire;
mod asset_project_flow_sample;
mod asset_scene;
mod asset_tests;
mod asset_ui;
mod code_review_findings;
mod core_framework;
mod core_runtime_deactivation;
mod core_runtime_registration;
mod dynamic_scene_absorption;
mod export_build_plan;
mod export_build_plan_platform;
mod extension_registry_bridge;
mod historical_oversized_roots;
mod input_manager;
mod manifest_contributions;
mod module_layout;
mod native_live_host_tests;
mod native_plugin_loader;
mod picking;
mod render_graph_resources;
mod render_products;
mod rhi_command_list;
mod rhi_device_contract;
mod root_entries;
mod root_layout;
mod runtime_diagnostics;
mod runtime_plugin_package_manifest;
mod scene_asset_integration;
mod scene_component_structure;
mod scene_derived_state;
mod scene_dynamic_scene_root;
mod scene_dynamic_session;
mod scene_ecs_query;
mod scene_ecs_query_structure;
mod scene_ecs_reflect_foundation;
mod scene_ecs_schedule;
mod scene_ecs_systems;
mod scene_property_paths;
mod scene_render_extract;
mod scene_world_basics;
mod script_vm_tests;
mod shader_prewarm_manifest;
mod shader_prewarm_registry_revision;
mod status_output_expected_slices;
mod status_output_row_data;
mod ui_accessibility;
mod ui_accessibility_widget_actions;
mod ui_architecture;
mod ui_asset;
mod ui_asset_mui_web_form_style;
mod ui_asset_mui_web_mui_x_style;
mod ui_asset_mui_web_style;
mod ui_asset_surface_index;
mod ui_boundary;
mod ui_component_catalog;
mod ui_component_catalog_component_state;
mod ui_component_catalog_component_state_keyboard;
mod ui_component_catalog_material_foundation;
mod ui_event_routing;
mod ui_focus_navigation;
mod ui_layout_slots;
mod ui_material_layout;
mod ui_runtime_input_manager;
mod ui_runtime_input_ownership;
mod ui_runtime_input_reply_routes;
mod ui_runtime_window_event_abi;
mod ui_runtime_window_input_pump;
mod ui_shared_core;
mod ui_surface_dirty_domains;
mod ui_surface_frame_authority;
mod ui_taffy_layout_pass;
mod ui_template;
mod ui_v2_asset;
mod ui_widget_text_input_keyboard;

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
