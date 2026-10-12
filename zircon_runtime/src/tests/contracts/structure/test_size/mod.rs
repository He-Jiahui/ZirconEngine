//! 汇集文件预算与职责迁移的源码守卫，并统一磁盘读取和换行处理；断言依赖当前检出源码及文档，不能据此证明产品运行效果。
use super::{assert_contains_all, assert_contains_all_exact, repo_path, runtime_src_path};

#[path = "asset_artifact_store.rs"]
mod asset_artifact_store;
#[path = "asset_gltf_importer.rs"]
mod asset_gltf_importer;
#[path = "../../../structure/test_size/asset_gltf_primitive_fixtures.rs"]
mod asset_gltf_primitive_fixtures;
#[path = "asset_importer.rs"]
mod asset_importer;
#[path = "asset_mesh.rs"]
mod asset_mesh;
#[path = "asset_pipeline_manager.rs"]
mod asset_pipeline_manager;
#[path = "../../../structure/test_size/asset_project_example_vampire.rs"]
mod asset_project_example_vampire;
#[path = "../../../structure/test_size/asset_project_flow_sample.rs"]
mod asset_project_flow_sample;
#[path = "asset_scene.rs"]
mod asset_scene;
#[path = "asset.rs"]
mod asset_tests;
#[path = "asset_ui.rs"]
mod asset_ui;
#[path = "review.rs"]
mod code_review_findings;
#[path = "core_framework.rs"]
mod core_framework;
#[path = "../../../structure/test_size/core_runtime_deactivation.rs"]
mod core_runtime_deactivation;
#[path = "core_runtime_register.rs"]
mod core_runtime_registration;
#[path = "../../../test_size/depth_prepass_pure_depth_product_migration.rs"]
mod depth_prepass_pure_depth_product_migration;
#[path = "../../../structure/test_size/dynamic_scene_absorption.rs"]
mod dynamic_scene_absorption;
#[path = "../../../structure/test_size/editor_pane_data_conversion.rs"]
mod editor_pane_data_conversion;
#[path = "../../../structure/test_size/editor_workbench_window_project.rs"]
mod editor_workbench_window_projection;
#[path = "export_build_plan.rs"]
mod export_build_plan;
#[path = "../../../structure/test_size/export_build_plan_platform.rs"]
mod export_build_plan_platform;
#[path = "../../../structure/test_size/extension_registry_bridge.rs"]
mod extension_registry_bridge;
#[path = "global_budget.rs"]
mod global_budget;
#[path = "../../../structure/test_size/historical_oversized_roots.rs"]
mod historical_oversized_roots;
#[path = "hub_project_actions.rs"]
mod hub_project_actions;
#[path = "hub_runtime_state.rs"]
mod hub_runtime_state;
#[path = "hub_view_model.rs"]
mod hub_view_model;
#[path = "input_manager.rs"]
mod input_manager;
#[path = "manifest_contributions.rs"]
mod manifest_contributions;
#[path = "../../../test_size/material_custom_shading_model_runtime.rs"]
mod material_custom_shading_model_runtime;
#[path = "../../../test_size/mesh_pipeline_variant_cache_owner.rs"]
mod mesh_pipeline_variant_cache_owner;
#[path = "module_layout.rs"]
mod module_layout;
#[path = "../../../structure/test_size/morph_geometry_source_selection.rs"]
mod morph_geometry_source_selection;
#[path = "../../../structure/test_size/morph_payload_projection.rs"]
mod morph_payload_projection;
#[path = "../../../structure/test_size/morph_payload_slot_indexing.rs"]
mod morph_payload_slot_indexing;
#[path = "../../../structure/test_size/morph_storage_buffers_upload.rs"]
mod morph_storage_buffers_upload;
#[path = "../../../structure/test_size/naming_boundary_asset_dynamic.rs"]
mod naming_boundary_asset_dynamic;
#[path = "../../../test_size/naming_boundary_asset_dynamic_asset_watch.rs"]
mod naming_boundary_asset_dynamic_asset_watch;
#[path = "../../../naming_boundary_asset_dynamic_dynamic_api_vampire.rs"]
mod naming_boundary_asset_dynamic_dynamic_api_vampire;
#[path = "../../../naming_boundary_asset_dynamic_scene_ecs_queries.rs"]
mod naming_boundary_asset_dynamic_scene_ecs_queries;
#[path = "../../../test_size/naming_boundary_asset_schema_material.rs"]
mod naming_boundary_asset_schema_material;
#[path = "../../../structure/test_size/naming_boundary_banned_names.rs"]
mod naming_boundary_banned_names;
#[path = "../../../naming_boundary_banned_names_global_modules.rs"]
mod naming_boundary_banned_names_global_modules;
#[path = "../../../naming_boundary_banned_names_graphics_construct.rs"]
mod naming_boundary_banned_names_graphics_construction;
#[path = "../../../structure/test_size/naming_boundary_core_framework.rs"]
mod naming_boundary_core_framework;
#[path = "../../../naming_boundary_core_framework_render_fixtures.rs"]
mod naming_boundary_core_framework_render_fixtures;
#[path = "../../../naming_boundary_core_framework_render_layer.rs"]
mod naming_boundary_core_framework_render_layer;
#[path = "../../../structure/test_size/naming_boundary_core_scene.rs"]
mod naming_boundary_core_scene;
#[path = "../../../structure/test_size/naming_boundary_core_scene_ecs.rs"]
mod naming_boundary_core_scene_ecs;
#[path = "../../../test_size/naming_boundary_core_scene_render_layer.rs"]
mod naming_boundary_core_scene_render_layer;
#[path = "../../../test_size/naming_boundary_core_scene_runtime_state.rs"]
mod naming_boundary_core_scene_runtime_state;
#[path = "../../../structure/test_size/naming_boundary_graphics.rs"]
mod naming_boundary_graphics;
#[path = "../../../test_size/naming_boundary_graphics_gpu_model.rs"]
mod naming_boundary_graphics_gpu_model;
#[path = "../../../test_size/naming_boundary_graphics_offscreen_target.rs"]
mod naming_boundary_graphics_offscreen_target;
#[path = "../../../test_size/naming_boundary_graphics_render_framework.rs"]
mod naming_boundary_graphics_render_framework;
#[path = "../../../test_size/naming_boundary_graphics_resource_streamer.rs"]
mod naming_boundary_graphics_resource_streamer;
#[path = "../../../structure/test_size/naming_boundary_hub_raw_text.rs"]
mod naming_boundary_hub_raw_text;
#[path = "../../../test_size/naming_boundary_input_mouse_wheel.rs"]
mod naming_boundary_input_mouse_wheel;
#[path = "../../../structure/test_size/naming_boundary_net_http.rs"]
mod naming_boundary_net_http;
#[path = "../../../test_size/naming_boundary_plugin_static_manifest.rs"]
mod naming_boundary_plugin_static_manifest;
#[path = "../../../test_size/naming_boundary_scene_tests_ecs_systems.rs"]
mod naming_boundary_scene_tests_ecs_systems;
#[path = "../../../test_size/naming_boundary_ui_platform_input.rs"]
mod naming_boundary_ui_platform_input;
#[path = "native_live_host.rs"]
mod native_live_host_tests;
#[path = "loader.rs"]
mod native_plugin_loader;
#[path = "picking.rs"]
mod picking;
#[path = "priority_plan_docs.rs"]
mod priority_plan_docs;
#[path = "render_graph_resources.rs"]
mod render_graph_resources;
#[path = "../../../test_size/render_plan08_staged_prewarm_product_sweep.rs"]
mod render_plan08_staged_prewarm_product_sweep;
#[path = "../../../render_plan08_three_shading_models_forward_deferred.rs"]
mod render_plan08_three_shading_models_forward_deferred_parity;
#[path = "../../../structure/test_size/render_product_mesh_cache_morph.rs"]
mod render_product_mesh_cache_morph;
#[path = "render_products.rs"]
mod render_products;
#[path = "rhi_command_list.rs"]
mod rhi_command_list;
#[path = "rhi_device_contract.rs"]
mod rhi_device_contract;
#[path = "root_entries.rs"]
mod root_entries;
#[path = "root_layout.rs"]
mod root_layout;
#[path = "../../../test_size/runtime_07_perf_hotspots_owner_budget.rs"]
mod runtime_07_performance_hotspots_owner_budget;
#[path = "../../../runtime_07_perf_hotspots_owner_budget_large_file.rs"]
mod runtime_07_performance_hotspots_owner_budget_large_file;
#[path = "../../../runtime_07_perf_hotspots_owner_budget_mirror_docs.rs"]
mod runtime_07_performance_hotspots_owner_budget_mirror_docs;
#[path = "runtime_diagnostics.rs"]
mod runtime_diagnostics;
#[path = "../../../structure/test_size/runtime_plugin_catalog_features.rs"]
mod runtime_plugin_catalog_features;
#[path = "../../../structure/test_size/runtime_plugin_lifecycle.rs"]
mod runtime_plugin_lifecycle;
#[path = "../../../structure/test_size/runtime_plugin_package_manifest.rs"]
mod runtime_plugin_package_manifest;
#[path = "../../../structure/test_size/scene_asset_integration.rs"]
mod scene_asset_integration;
#[path = "../../../structure/test_size/scene_component_structure.rs"]
mod scene_component_structure;
#[path = "scene_derived_state.rs"]
mod scene_derived_state;
#[path = "../../../structure/test_size/scene_dynamic_scene_root.rs"]
mod scene_dynamic_scene_root;
#[path = "scene_dynamic_session.rs"]
mod scene_dynamic_session;
#[path = "scene_ecs_query.rs"]
mod scene_ecs_query;
#[path = "../../../structure/test_size/scene_ecs_query_structure.rs"]
mod scene_ecs_query_structure;
#[path = "../../../structure/test_size/scene_ecs_reflect_foundation.rs"]
mod scene_ecs_reflect_foundation;
#[path = "scene_ecs_schedule.rs"]
mod scene_ecs_schedule;
#[path = "scene_ecs_systems.rs"]
mod scene_ecs_systems;
#[path = "scene_property_paths.rs"]
mod scene_property_paths;
#[path = "scene_render_extract.rs"]
mod scene_render_extract;
#[path = "scene_world_basics.rs"]
mod scene_world_basics;
#[path = "script_vm.rs"]
mod script_vm_tests;
#[path = "../../../test_size/shader_prewarm_acceptance_contract.rs"]
mod shader_prewarm_acceptance_contract;
#[path = "../../../test_size/shader_prewarm_asset_root_plan_visibility.rs"]
mod shader_prewarm_asset_root_plan_visibility;
#[path = "../../../test_size/shader_prewarm_cache_artifact_contract.rs"]
mod shader_prewarm_cache_artifact_contract;
#[path = "../../../test_size/shader_prewarm_live_resource_registry.rs"]
mod shader_prewarm_live_resource_registry;
#[path = "../../../structure/test_size/shader_prewarm_manifest.rs"]
mod shader_prewarm_manifest;
#[path = "../../../test_size/shader_prewarm_permutation_registry.rs"]
mod shader_prewarm_permutation_registry;
#[path = "../../../shader_prewarm_permutation_registry_auto_export.rs"]
mod shader_prewarm_permutation_registry_auto_export;
#[path = "../../../shader_prewarm_plugin_asset_roots_auto_export.rs"]
mod shader_prewarm_plugin_asset_roots_auto_export;
#[path = "../../../test_size/shader_prewarm_plugin_geometry_source_desc.rs"]
mod shader_prewarm_plugin_geometry_source_descriptor;
#[path = "../../../shader_prewarm_plugin_permutation_registry_auto.rs"]
mod shader_prewarm_plugin_permutation_registry_auto_export;
#[path = "../../../test_size/shader_prewarm_plugin_shading_model_desc.rs"]
mod shader_prewarm_plugin_shading_model_descriptor;
#[path = "../../../shader_prewarm_project_asset_roots_auto_export.rs"]
mod shader_prewarm_project_asset_roots_auto_export;
#[path = "../../../shader_prewarm_project_plugin_registry_auto_export.rs"]
mod shader_prewarm_project_plugin_registry_auto_export;
#[path = "../../../shader_prewarm_project_plugin_registry_export_file.rs"]
mod shader_prewarm_project_plugin_registry_export_file;
#[path = "../../../shader_prewarm_project_plugin_registry_live_asset.rs"]
mod shader_prewarm_project_plugin_registry_live_asset_roots;
#[path = "../../../project_plugin_registry_material_passes_staged_cache.rs"]
mod shader_prewarm_project_plugin_registry_material_passes_staged_cache;
#[path = "../../../prewarm_project_plugin_registry_product_staged_cache.rs"]
mod shader_prewarm_project_plugin_registry_product_staged_cache;
#[path = "../../../project_plugin_registry_production_cli_dry_run.rs"]
mod shader_prewarm_project_plugin_registry_production_cli_dry_run;
#[path = "../../../shader_prewarm_project_plugin_registry_production.rs"]
mod shader_prewarm_project_plugin_registry_production_cli_selection;
#[path = "../../../prewarm_project_plugin_registry_production_command.rs"]
mod shader_prewarm_project_plugin_registry_production_command;
#[path = "../../../prewarm_project_plugin_registry_production_fixture.rs"]
mod shader_prewarm_project_plugin_registry_production_fixture;
#[path = "../../../prewarm_project_plugin_registry_production_live_wgpu.rs"]
mod shader_prewarm_project_plugin_registry_production_live_wgpu;
#[path = "../../../project_plugin_registry_production_wrapper_no_proxy.rs"]
mod shader_prewarm_project_plugin_registry_production_wrapper_no_proxy;
#[path = "../../../shader_prewarm_project_plugin_registry.rs"]
mod shader_prewarm_project_plugin_registry_production_wrapper_orchestration;
#[path = "../../../shader_prewarm_project_plugin_registry_report_source.rs"]
mod shader_prewarm_project_plugin_registry_report_source;
#[path = "../../../project_plugin_registry_runtime_staged_cache_hit.rs"]
mod shader_prewarm_project_plugin_registry_runtime_staged_cache_hit;
#[path = "../../../test_size/shader_prewarm_registry_auto_export.rs"]
mod shader_prewarm_registry_auto_export;
#[path = "../../../structure/test_size/shader_prewarm_registry_revision.rs"]
mod shader_prewarm_registry_revision;
#[path = "../../../test_size/shader_prewarm_report_dimension_contract.rs"]
mod shader_prewarm_report_dimension_contract;
#[path = "../../../shader_prewarm_resource_registry_export_contract.rs"]
mod shader_prewarm_resource_registry_export_contract;
#[path = "../../../shader_prewarm_resource_registry_multi_root_dedupe.rs"]
mod shader_prewarm_resource_registry_multi_root_dedupe;
#[path = "../../../shader_prewarm_resource_registry_report_correlation.rs"]
mod shader_prewarm_resource_registry_report_correlation;
#[path = "../../../shader_prewarm_source_provenance_report_contract.rs"]
mod shader_prewarm_source_provenance_report_contract;
#[path = "../../../test_size/shader_prewarm_source_provenance_summary.rs"]
mod shader_prewarm_source_provenance_summary;
#[path = "../../../shader_prewarm_staged_wgpu_handoff_command_contract.rs"]
mod shader_prewarm_staged_wgpu_handoff_command_contract;
#[path = "../../../test_size/shader_prewarm_wgpu_module_validate.rs"]
mod shader_prewarm_wgpu_module_validation;
#[path = "../../../test_size/shader_prewarm_wgpu_pipeline_validate.rs"]
mod shader_prewarm_wgpu_pipeline_validation;
#[path = "../../../test_size/shader_prewarm_wgpu_report_contract.rs"]
mod shader_prewarm_wgpu_report_contract;
#[path = "../../../shader_prewarm_wgpu_validate_report_summary.rs"]
mod shader_prewarm_wgpu_validation_report_summary;
#[path = "../../../structure/test_size/shader_surface_source_contract.rs"]
mod shader_surface_source_contract;
#[path = "../../../test_size/taa_reactive_shader_pass_identity.rs"]
mod taa_reactive_shader_pass_identity;
#[path = "ui_accessibility.rs"]
mod ui_accessibility;
#[path = "../../../structure/test_size/ui_accessibility_widget_actions.rs"]
mod ui_accessibility_widget_actions;
#[path = "ui_architecture.rs"]
mod ui_architecture;
#[path = "ui_asset.rs"]
mod ui_asset;
#[path = "../../../structure/test_size/ui_asset_mui_web_form_style.rs"]
mod ui_asset_mui_web_form_style;
#[path = "../../../structure/test_size/ui_asset_mui_web_mui_x_style.rs"]
mod ui_asset_mui_web_mui_x_style;
#[path = "ui_asset_mui_web_style.rs"]
mod ui_asset_mui_web_style;
#[path = "ui_asset_surface_index.rs"]
mod ui_asset_surface_index;
#[path = "ui_boundary.rs"]
mod ui_boundary;
#[path = "ui_component_catalog.rs"]
mod ui_component_catalog;
#[path = "../../../test_size/ui_component_catalog_component_state.rs"]
mod ui_component_catalog_component_state;
#[path = "../../../ui_component_catalog_component_state_keyboard.rs"]
mod ui_component_catalog_component_state_keyboard;
#[path = "../../../test_size/ui_component_catalog_material_foundation.rs"]
mod ui_component_catalog_material_foundation;
#[path = "ui_event_routing.rs"]
mod ui_event_routing;
#[path = "ui_focus_navigation.rs"]
mod ui_focus_navigation;
#[path = "ui_layout_slots.rs"]
mod ui_layout_slots;
#[path = "ui_material_layout.rs"]
mod ui_material_layout;
#[path = "../../../structure/test_size/ui_runtime_input_manager.rs"]
mod ui_runtime_input_manager;
#[path = "../../../structure/test_size/ui_runtime_input_ownership.rs"]
mod ui_runtime_input_ownership;
#[path = "../../../structure/test_size/ui_runtime_input_reply_routes.rs"]
mod ui_runtime_input_reply_routes;
#[path = "../../../structure/test_size/ui_runtime_window_event_abi.rs"]
mod ui_runtime_window_event_abi;
#[path = "../../../structure/test_size/ui_runtime_window_input_pump.rs"]
mod ui_runtime_window_input_pump;
#[path = "ui_shared_core.rs"]
mod ui_shared_core;
#[path = "../../../structure/test_size/ui_surface_dirty_domains.rs"]
mod ui_surface_dirty_domains;
#[path = "../../../structure/test_size/ui_surface_frame_authority.rs"]
mod ui_surface_frame_authority;
#[path = "ui_taffy_layout_pass.rs"]
mod ui_taffy_layout_pass;
#[path = "ui_template.rs"]
mod ui_template;
#[path = "ui_text_layout.rs"]
mod ui_text_layout;
#[path = "ui_v2_asset.rs"]
mod ui_v2_asset;
#[path = "../../../structure/test_size/ui_widget_text_input_keyboard.rs"]
mod ui_widget_text_input_keyboard;
#[path = "../../../test_size/virtual_geometry_asset_payload_decode.rs"]
mod virtual_geometry_asset_payload_decode;
#[path = "../../../test_size/virtual_geometry_cluster_payload_upload.rs"]
mod virtual_geometry_cluster_payload_upload;
#[path = "../../../test_size/virtual_geometry_meshlet_vertex_ordinal.rs"]
mod virtual_geometry_meshlet_vertex_ordinal;
#[path = "../../../virtual_geometry_page_cluster_shader_bindings.rs"]
mod virtual_geometry_page_cluster_shader_bindings;
#[path = "../../../test_size/virtual_geometry_product_draw_source.rs"]
mod virtual_geometry_product_draw_source;
#[path = "../../../test_size/virtual_geometry_resident_buffers_upload.rs"]
mod virtual_geometry_resident_buffers_upload;

fn read_runtime_src(relative: &str) -> String {
    normalize_line_endings(
        std::fs::read_to_string(runtime_src_path(relative))
            .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}")),
    )
}

fn read_runtime_src_with_children(parent: &str, child_dir: &str) -> String {
    let mut source = read_runtime_src(parent);
    let child_dir = runtime_src_path(child_dir);
    let mut child_paths = std::fs::read_dir(&child_dir)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read runtime source directory {}: {error}",
                child_dir.display()
            )
        })
        .map(|entry| {
            entry
                .expect("runtime status-row child entry should be readable")
                .path()
        })
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
        .collect::<Vec<_>>();
    child_paths.sort();
    for path in child_paths {
        source.push('\n');
        source.push_str(&std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("failed to read runtime source {}: {error}", path.display())
        }));
    }
    normalize_line_endings(source)
}

fn read_repo(relative: &str) -> String {
    normalize_line_endings(
        std::fs::read_to_string(repo_path(relative))
            .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}")),
    )
}

fn read_zircon_build_sources() -> String {
    format!(
        "{}\n{}",
        read_repo("tools/build/zircon_build.py"),
        read_repo("tools/build/zircon_build_config.py")
    )
}

fn normalize_line_endings(source: String) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}
