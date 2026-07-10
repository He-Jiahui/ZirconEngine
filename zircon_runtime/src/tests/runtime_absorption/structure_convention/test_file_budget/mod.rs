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
mod depth_prepass_pure_depth_product_migration;
mod dynamic_scene_absorption;
mod editor_pane_data_conversion;
mod editor_workbench_window_projection;
mod export_build_plan;
mod export_build_plan_platform;
mod extension_registry_bridge;
mod global_budget;
mod historical_oversized_roots;
mod hub_project_actions;
mod hub_runtime_state;
mod hub_view_model;
mod input_manager;
mod manifest_contributions;
mod material_custom_shading_model_runtime;
mod mesh_pipeline_variant_cache_owner;
mod module_layout;
mod morph_geometry_source_selection;
mod morph_payload_projection;
mod morph_payload_slot_indexing;
mod morph_storage_buffers_upload;
mod naming_boundary_asset_dynamic;
mod naming_boundary_asset_dynamic_asset_watch;
mod naming_boundary_asset_dynamic_dynamic_api_vampire;
mod naming_boundary_asset_dynamic_scene_ecs_queries;
mod naming_boundary_asset_schema_material;
mod naming_boundary_banned_names;
mod naming_boundary_banned_names_global_modules;
mod naming_boundary_banned_names_graphics_construction;
mod naming_boundary_core_framework;
mod naming_boundary_core_framework_render_fixtures;
mod naming_boundary_core_framework_render_layer;
mod naming_boundary_core_scene;
mod naming_boundary_core_scene_ecs;
mod naming_boundary_core_scene_render_layer;
mod naming_boundary_core_scene_runtime_state;
mod naming_boundary_graphics;
mod naming_boundary_graphics_gpu_model;
mod naming_boundary_graphics_offscreen_target;
mod naming_boundary_graphics_render_framework;
mod naming_boundary_graphics_resource_streamer;
mod naming_boundary_hub_raw_text;
mod naming_boundary_input_mouse_wheel;
mod naming_boundary_net_http;
mod naming_boundary_plugin_static_manifest;
mod naming_boundary_scene_tests_ecs_systems;
mod naming_boundary_ui_platform_input;
mod native_live_host_tests;
mod native_plugin_loader;
mod picking;
mod priority_plan_docs;
mod render_graph_resources;
mod render_plan08_staged_prewarm_product_sweep;
mod render_plan08_three_shading_models_forward_deferred_parity;
mod render_product_mesh_cache_morph;
mod render_products;
mod rhi_command_list;
mod rhi_device_contract;
mod root_entries;
mod root_layout;
mod runtime_07_performance_hotspots_owner_budget;
mod runtime_07_performance_hotspots_owner_budget_large_file;
mod runtime_07_performance_hotspots_owner_budget_mirror_docs;
mod runtime_diagnostics;
mod runtime_plugin_catalog_features;
mod runtime_plugin_lifecycle;
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
mod shader_prewarm_acceptance_contract;
mod shader_prewarm_asset_root_plan_visibility;
mod shader_prewarm_cache_artifact_contract;
mod shader_prewarm_live_resource_registry;
mod shader_prewarm_manifest;
mod shader_prewarm_permutation_registry;
mod shader_prewarm_permutation_registry_auto_export;
mod shader_prewarm_plugin_asset_roots_auto_export;
mod shader_prewarm_plugin_geometry_source_descriptor;
mod shader_prewarm_plugin_permutation_registry_auto_export;
mod shader_prewarm_plugin_shading_model_descriptor;
mod shader_prewarm_project_asset_roots_auto_export;
mod shader_prewarm_project_plugin_registry_auto_export;
mod shader_prewarm_project_plugin_registry_export_file;
mod shader_prewarm_project_plugin_registry_live_asset_roots;
mod shader_prewarm_project_plugin_registry_material_passes_staged_cache;
mod shader_prewarm_project_plugin_registry_product_staged_cache;
mod shader_prewarm_project_plugin_registry_production_cli_dry_run;
mod shader_prewarm_project_plugin_registry_production_cli_selection;
mod shader_prewarm_project_plugin_registry_production_command;
mod shader_prewarm_project_plugin_registry_production_fixture;
mod shader_prewarm_project_plugin_registry_production_live_wgpu;
mod shader_prewarm_project_plugin_registry_production_wrapper_no_proxy;
mod shader_prewarm_project_plugin_registry_production_wrapper_orchestration;
mod shader_prewarm_project_plugin_registry_report_source;
mod shader_prewarm_project_plugin_registry_runtime_staged_cache_hit;
mod shader_prewarm_registry_auto_export;
mod shader_prewarm_registry_revision;
mod shader_prewarm_report_dimension_contract;
mod shader_prewarm_resource_registry_export_contract;
mod shader_prewarm_resource_registry_multi_root_dedupe;
mod shader_prewarm_resource_registry_report_correlation;
mod shader_prewarm_source_provenance_report_contract;
mod shader_prewarm_source_provenance_summary;
mod shader_prewarm_staged_wgpu_handoff_command_contract;
mod shader_prewarm_wgpu_module_validation;
mod shader_prewarm_wgpu_pipeline_validation;
mod shader_prewarm_wgpu_report_contract;
mod shader_prewarm_wgpu_validation_report_summary;
mod status_output_expected_slices;
mod status_output_row_data;
mod taa_reactive_shader_pass_identity;
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
mod ui_text_layout;
mod ui_v2_asset;
mod ui_widget_text_input_keyboard;
mod virtual_geometry_asset_payload_decode;
mod virtual_geometry_cluster_payload_upload;
mod virtual_geometry_meshlet_vertex_ordinal;
mod virtual_geometry_page_cluster_shader_bindings;
mod virtual_geometry_product_draw_source;
mod virtual_geometry_resident_buffers_upload;

fn read_runtime_src(relative: &str) -> String {
    normalize_line_endings(
        std::fs::read_to_string(runtime_src_path(relative))
            .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}")),
    )
}

fn ui_tests_first_status_row_source() -> String {
    read_runtime_src_with_children(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first",
    )
}

fn ui_tests_second_status_row_source() -> String {
    read_runtime_src_with_children(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second",
    )
}

fn read_runtime_src_with_children(parent: &str, child_dir: &str) -> String {
    let mut source = read_runtime_src(parent);
    let child_dir = runtime_src_path(child_dir);
    let mut child_paths = std::fs::read_dir(&child_dir)
        .unwrap_or_else(|error| panic!("failed to read runtime source directory {}: {error}", child_dir.display()))
        .map(|entry| entry.expect("runtime status-row child entry should be readable").path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
        .collect::<Vec<_>>();
    child_paths.sort();
    for path in child_paths {
        source.push('\n');
        source.push_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read runtime source {}: {error}", path.display())),
        );
    }
    normalize_line_endings(source)
}

fn read_runtime_src_route_tree(parent: &str) -> String {
    let child_dir = parent
        .strip_suffix(".rs")
        .unwrap_or_else(|| panic!("runtime route source should end in .rs: {parent}"));
    read_runtime_src_with_children(parent, child_dir)
}

fn read_repo(relative: &str) -> String {
    normalize_line_endings(
        std::fs::read_to_string(repo_path(relative))
            .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}")),
    )
}

fn normalize_line_endings(source: String) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}
