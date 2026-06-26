use super::{assert_contains_all, repo_path, runtime_src_path};

#[path = "production_file_budget/asset_cache_payload.rs"]
mod asset_cache_payload;
#[path = "production_file_budget/asset_gltf_labeled_subassets.rs"]
mod asset_gltf_labeled_subassets;
#[path = "production_file_budget/asset_project_scan_import.rs"]
mod asset_project_scan_import;
#[path = "production_file_budget/compiled_graph_cache_tests.rs"]
mod compiled_graph_cache_tests;
#[path = "production_file_budget/core_runtime_service_lists.rs"]
mod core_runtime_service_lists;
#[path = "production_file_budget/dynamic_api_session_profile.rs"]
mod dynamic_api_session_profile;
#[path = "production_file_budget/dynamic_api_session_registry.rs"]
mod dynamic_api_session_registry;
#[path = "production_file_budget/hzb_occlusion_culler.rs"]
mod hzb_occlusion_culler;
#[path = "production_file_budget/m4_behavior_postprocess_tests.rs"]
mod m4_behavior_postprocess_tests;
#[path = "production_file_budget/material_asset.rs"]
mod material_asset;
#[path = "production_file_budget/material_runtime_pbr_projection_tests.rs"]
mod material_runtime_pbr_projection_tests;
#[path = "production_file_budget/mesh_asset.rs"]
mod mesh_asset;
#[path = "production_file_budget/module_layout.rs"]
mod module_layout;
#[path = "production_file_budget/native_host_api_adapter.rs"]
mod native_host_api_adapter;
#[path = "production_file_budget/render_backend_types.rs"]
mod render_backend_types;
#[path = "production_file_budget/render_build_virtual_geometry_debug_snapshot.rs"]
mod render_build_virtual_geometry_debug_snapshot;
#[path = "production_file_budget/render_extend_pending_draws_for_mesh_instance.rs"]
mod render_extend_pending_draws_for_mesh_instance;
#[path = "production_file_budget/render_frame_extract_geometry.rs"]
mod render_frame_extract_geometry;
#[path = "production_file_budget/render_frame_submission_context_tests.rs"]
mod render_frame_submission_context_tests;
#[path = "production_file_budget/render_framework_bridge_tests.rs"]
mod render_framework_bridge_tests;
#[path = "production_file_budget/render_gpu_scene_tests.rs"]
mod render_gpu_scene_tests;
#[path = "production_file_budget/render_gpu_texture_from_asset_tests.rs"]
mod render_gpu_texture_from_asset_tests;
#[path = "production_file_budget/render_graph_builder_compile.rs"]
mod render_graph_builder_compile;
#[path = "production_file_budget/render_graph_materialization_tests.rs"]
mod render_graph_materialization_tests;
#[path = "production_file_budget/render_material_management_tests.rs"]
mod render_material_management_tests;
#[path = "production_file_budget/render_material_product_debug_counts_tests.rs"]
mod render_material_product_debug_counts_tests;
#[path = "production_file_budget/render_material_readiness_report_tests.rs"]
mod render_material_readiness_report_tests;
#[path = "production_file_budget/render_mesh_build_draws_build.rs"]
mod render_mesh_build_draws_build;
#[path = "production_file_budget/render_mesh_build_draws_skinning_tests.rs"]
mod render_mesh_build_draws_skinning_tests;
#[path = "production_file_budget/render_pass_executor_registry_tests.rs"]
mod render_pass_executor_registry_tests;
#[path = "production_file_budget/render_pass_gpu_context_mesh_command_lists.rs"]
mod render_pass_gpu_context_mesh_command_lists;
#[path = "production_file_budget/render_pipeline_asset_compile_tests.rs"]
mod render_pipeline_asset_compile_tests;
#[path = "production_file_budget/render_pipeline_compile_monolith_tests.rs"]
mod render_pipeline_compile_monolith_tests;
#[path = "production_file_budget/render_plugin_feature_compile_particle_tests.rs"]
mod render_plugin_feature_compile_particle_tests;
#[path = "production_file_budget/render_post_process_screen_space_reflection_tests.rs"]
mod render_post_process_screen_space_reflection_tests;
#[path = "production_file_budget/render_product_anti_alias_focused_tests.rs"]
mod render_product_anti_alias_focused_tests;
#[path = "production_file_budget/render_product_mesh_cache_virtual_geometry_tests.rs"]
mod render_product_mesh_cache_virtual_geometry_tests;
#[path = "production_file_budget/render_product_post_process_motion_blur_tests.rs"]
mod render_product_post_process_motion_blur_tests;
#[path = "production_file_budget/render_product_shadow_captures_directional_tests.rs"]
mod render_product_shadow_captures_directional_tests;
#[path = "production_file_budget/render_product_shadows_many_point_lights_tests.rs"]
mod render_product_shadows_many_point_lights_tests;
#[path = "production_file_budget/render_product_submit_profiles_tests.rs"]
mod render_product_submit_profiles_tests;
#[path = "production_file_budget/render_project_render_quality_tests.rs"]
mod render_project_render_quality_tests;
#[path = "production_file_budget/render_project_scene_products_tests.rs"]
mod render_project_scene_products_tests;
#[path = "production_file_budget/render_renderer_data_asset_compile_tests.rs"]
mod render_renderer_data_asset_compile_tests;
#[path = "production_file_budget/render_scene_world.rs"]
mod render_scene_world;
#[path = "production_file_budget/render_shader_geometry_source_descriptor.rs"]
mod render_shader_geometry_source_descriptor;
#[path = "production_file_budget/render_shader_template_assembly.rs"]
mod render_shader_template_assembly;
#[path = "production_file_budget/render_shadow.rs"]
mod render_shadow;
#[path = "production_file_budget/render_shadow_atlas_plan_tests.rs"]
mod render_shadow_atlas_plan_tests;
#[path = "production_file_budget/render_sprite_build_vertices_tests.rs"]
mod render_sprite_build_vertices_tests;
#[path = "production_file_budget/render_stats_graph.rs"]
mod render_stats_graph;
#[path = "production_file_budget/render_stats_product_tests.rs"]
mod render_stats_product_tests;
#[path = "production_file_budget/render_submit_camera_loop.rs"]
mod render_submit_camera_loop;
#[path = "production_file_budget/render_surface_targets_texture_target_tests.rs"]
mod render_surface_targets_texture_target_tests;
#[path = "production_file_budget/render_ui_screen_space_render.rs"]
mod render_ui_screen_space_render;
#[path = "production_file_budget/render_ui_sdf_atlas_tests.rs"]
mod render_ui_sdf_atlas_tests;
#[path = "production_file_budget/render_ui_sdf_render.rs"]
mod render_ui_sdf_render;
#[path = "production_file_budget/render_update_base_stats.rs"]
mod render_update_base_stats;
#[path = "production_file_budget/render_update_base_stats_post_process_diagnostics.rs"]
mod render_update_base_stats_post_process_diagnostics;
#[path = "production_file_budget/render_vg_debug_snapshot_streams.rs"]
mod render_vg_debug_snapshot_streams;
#[path = "production_file_budget/render_visibility_context_construct_tests.rs"]
mod render_visibility_context_construct_tests;
#[path = "production_file_budget/render_visibility_virtual_geometry_tests.rs"]
mod render_visibility_virtual_geometry_tests;
#[path = "production_file_budget/rhi_device_handles.rs"]
mod rhi_device_handles;
#[path = "production_file_budget/rhi_wgpu_command_validation.rs"]
mod rhi_wgpu_command_validation;
#[path = "production_file_budget/rhi_wgpu_ui_surface_geometry.rs"]
mod rhi_wgpu_ui_surface_geometry;
#[path = "production_file_budget/rhi_wgpu_ui_surface_render_setup.rs"]
mod rhi_wgpu_ui_surface_render_setup;
#[path = "production_file_budget/scene_components.rs"]
mod scene_components;
#[path = "production_file_budget/scene_fixed_lights.rs"]
mod scene_fixed_lights;
#[path = "production_file_budget/scene_world_project_io.rs"]
mod scene_world_project_io;
#[path = "production_file_budget/scene_world_property_access.rs"]
mod scene_world_property_access;
#[path = "production_file_budget/scene_world_render_lights.rs"]
mod scene_world_render_lights;
#[path = "production_file_budget/texture_descriptor_settings.rs"]
mod texture_descriptor_settings;
#[path = "production_file_budget/ui_accessibility_extract.rs"]
mod ui_accessibility_extract;
#[path = "production_file_budget/ui_component_catalog_editor_showcase.rs"]
mod ui_component_catalog_editor_showcase;
#[path = "production_file_budget/ui_component_state_reducer_keyboard_menu.rs"]
mod ui_component_state_reducer_keyboard_menu;
#[path = "production_file_budget/ui_component_state_reducer_tree_view.rs"]
mod ui_component_state_reducer_tree_view;
#[path = "production_file_budget/ui_dispatch_input_manager_tests.rs"]
mod ui_dispatch_input_manager_tests;
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
