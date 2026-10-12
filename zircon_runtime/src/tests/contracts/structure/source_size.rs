//! 聚合生产文件职责拆分的结构测试；子测试检查源码和文档锚点，不执行被读取的生产路径。
use super::{assert_contains_all, assert_contains_all_exact, repo_path, runtime_src_path};

#[path = "source_size/asset_cache_payload.rs"]
mod asset_cache_payload;
#[path = "../../structure/source_size/asset_gltf_labeled_subassets.rs"]
mod asset_gltf_labeled_subassets;
#[path = "../../structure/source_size/asset_project_scan_import.rs"]
mod asset_project_scan_import;
#[path = "source_size/compiled_graph_cache.rs"]
mod compiled_graph_cache_tests;
#[path = "../../structure/source_size/core_runtime_service_lists.rs"]
mod core_runtime_service_lists;
#[path = "../../structure/source_size/dynamic_api_session_profile.rs"]
mod dynamic_api_session_profile;
#[path = "../../structure/source_size/dynamic_api_session_registry.rs"]
mod dynamic_api_session_registry;
#[path = "../../structure/source_size/dynamic_api_shader_prewarm.rs"]
mod dynamic_api_shader_prewarm_tests;
#[path = "source_size/font_database_descs.rs"]
mod font_database_descriptors;
#[path = "source_size/global_budget.rs"]
mod global_budget;
#[path = "source_size/hzb_occlusion_culler.rs"]
mod hzb_occlusion_culler;
#[path = "../../structure/source_size/m4_behavior_postprocess.rs"]
mod m4_behavior_postprocess_tests;
#[path = "source_size/material_asset.rs"]
mod material_asset;
#[path = "../../structure/source_size/material_runtime_pbr_project.rs"]
mod material_runtime_pbr_projection_tests;
#[path = "source_size/mesh_asset.rs"]
mod mesh_asset;
#[path = "source_size/module_layout.rs"]
mod module_layout;
#[path = "../../structure/source_size/native_host_api_adapter.rs"]
mod native_host_api_adapter;
#[path = "../../structure/source_size/plugin_bridge_table_reports.rs"]
mod plugin_bridge_table_reports;
#[path = "../../structure/source_size/render_asset_residency_manager.rs"]
mod render_asset_residency_manager;
#[path = "source_size/render_backend_types.rs"]
mod render_backend_types;
#[path = "../../render_build_virtual_geometry_debug_snapshot.rs"]
mod render_build_virtual_geometry_debug_snapshot;
#[path = "../../render_extend_pending_draws_for_mesh_instance.rs"]
mod render_extend_pending_draws_for_mesh_instance;
#[path = "../../structure/source_size/render_frame_extract_geometry.rs"]
mod render_frame_extract_geometry;
#[path = "../../source_size/render_frame_submission_context.rs"]
mod render_frame_submission_context_tests;
#[path = "../../structure/source_size/render_framework_bridge.rs"]
mod render_framework_bridge_tests;
#[path = "source_size/render_gpu_scene.rs"]
mod render_gpu_scene_tests;
#[path = "../../structure/source_size/render_gpu_texture_from_asset.rs"]
mod render_gpu_texture_from_asset_tests;
#[path = "../../structure/source_size/render_graph_builder_compile.rs"]
mod render_graph_builder_compile;
#[path = "../../structure/source_size/render_graph_materialization.rs"]
mod render_graph_materialization_tests;
#[path = "../../structure/source_size/render_material_manage.rs"]
mod render_material_management_tests;
#[path = "../../source_size/render_material_product_debug_counts.rs"]
mod render_material_product_debug_counts_tests;
#[path = "../../source_size/render_material_readiness_report.rs"]
mod render_material_readiness_report_tests;
#[path = "../../structure/source_size/render_mesh_build_draws_build.rs"]
mod render_mesh_build_draws_build;
#[path = "../../source_size/render_mesh_build_draws_skinning.rs"]
mod render_mesh_build_draws_skinning_tests;
#[path = "../../structure/source_size/render_pass_executor_registry.rs"]
mod render_pass_executor_registry_tests;
#[path = "../../render_pass_gpu_context_mesh_command_lists.rs"]
mod render_pass_gpu_context_mesh_command_lists;
#[path = "../../structure/source_size/render_pipeline_asset_compile.rs"]
mod render_pipeline_asset_compile_tests;
#[path = "../../source_size/render_pipeline_compile_monolith.rs"]
mod render_pipeline_compile_monolith_tests;
#[path = "../../source_size/render_plugin_feature_compile_particle.rs"]
mod render_plugin_feature_compile_particle_tests;
#[path = "../../render_post_process_screen_space_reflection.rs"]
mod render_post_process_screen_space_reflection_tests;
#[path = "../../source_size/render_product_anti_alias_focused.rs"]
mod render_product_anti_alias_focused_tests;
#[path = "../../render_product_mesh_cache_virtual_geometry.rs"]
mod render_product_mesh_cache_virtual_geometry_tests;
#[path = "../../source_size/render_product_post_process_motion_blur.rs"]
mod render_product_post_process_motion_blur_tests;
#[path = "../../render_product_shadow_captures_directional.rs"]
mod render_product_shadow_captures_directional_tests;
#[path = "../../source_size/render_product_shadows_many_point_lights.rs"]
mod render_product_shadows_many_point_lights_tests;
#[path = "../../structure/source_size/render_product_submit_profiles.rs"]
mod render_product_submit_profiles_tests;
#[path = "../../structure/source_size/render_project_render_quality.rs"]
mod render_project_render_quality_tests;
#[path = "../../structure/source_size/render_project_scene_products.rs"]
mod render_project_scene_products_tests;
#[path = "../../source_size/render_renderer_data_asset_compile.rs"]
mod render_renderer_data_asset_compile_tests;
#[path = "source_size/render_scene_world.rs"]
mod render_scene_world;
#[path = "../../source_size/render_shader_geometry_source_desc.rs"]
mod render_shader_geometry_source_descriptor;
#[path = "source_size/shaders.rs"]
mod render_shader_template_assembly;
#[path = "source_size/render_shadow.rs"]
mod render_shadow;
#[path = "../../structure/source_size/render_shadow_atlas_plan.rs"]
mod render_shadow_atlas_plan_tests;
#[path = "../../structure/source_size/render_sprite_build_vertices.rs"]
mod render_sprite_build_vertices_tests;
#[path = "source_size/render_stats_graph.rs"]
mod render_stats_graph;
#[path = "source_size/render_stats_product.rs"]
mod render_stats_product_tests;
#[path = "../../structure/source_size/render_submit_camera_loop.rs"]
mod render_submit_camera_loop;
#[path = "../../source_size/render_surface_targets_texture_target.rs"]
mod render_surface_targets_texture_target_tests;
#[path = "../../source_size/render_system_texture_generate_owner.rs"]
mod render_system_texture_generation_owner;
#[path = "../../structure/source_size/render_ui_image_binding_cache.rs"]
mod render_ui_image_binding_cache;
#[path = "../../structure/source_size/render_ui_image_vertex_buffer.rs"]
mod render_ui_image_vertex_buffer;
#[path = "../../render_ui_native_bitmap_atlas_upload_reuse.rs"]
mod render_ui_native_bitmap_atlas_upload_reuse;
#[path = "../../source_size/render_ui_resource_upload_transaction.rs"]
mod render_ui_resource_upload_transaction;
#[path = "../../structure/source_size/render_ui_screen_space_render.rs"]
mod render_ui_screen_space_render;
#[path = "../../source_size/render_ui_screen_space_vertex_buffer.rs"]
mod render_ui_screen_space_vertex_buffer;
#[path = "source_size/render_ui_sdf_atlas.rs"]
mod render_ui_sdf_atlas_tests;
#[path = "source_size/render_ui_sdf_render.rs"]
mod render_ui_sdf_render;
#[path = "../../structure/source_size/render_ui_text_font_id_report.rs"]
mod render_ui_text_font_id_report;
#[path = "source_size/render_ui_text.rs"]
mod render_ui_text_tests;
#[path = "../../structure/source_size/render_update_base_stats.rs"]
mod render_update_base_stats;
#[path = "../../render_update_base_stats_post_process_diagnostics.rs"]
mod render_update_base_stats_post_process_diagnostics;
#[path = "../../source_size/render_vg_debug_snapshot_streams.rs"]
mod render_vg_debug_snapshot_streams;
#[path = "../../source_size/render_visibility_context_construct.rs"]
mod render_visibility_context_construct_tests;
#[path = "../../source_size/render_visibility_virtual_geometry.rs"]
mod render_visibility_virtual_geometry_tests;
#[path = "source_size/rhi_device_handles.rs"]
mod rhi_device_handles;
#[path = "../../structure/source_size/rhi_wgpu_command_validate.rs"]
mod rhi_wgpu_command_validation;
#[path = "../../structure/source_size/rhi_wgpu_device_command_list.rs"]
mod rhi_wgpu_device_command_list;
#[path = "../../structure/source_size/rhi_wgpu_ui_surface_geometry.rs"]
mod rhi_wgpu_ui_surface_geometry;
#[path = "../../source_size/rhi_wgpu_ui_surface_render_setup.rs"]
mod rhi_wgpu_ui_surface_render_setup;
#[path = "source_size/scene_components.rs"]
mod scene_components;
#[path = "source_size/scene_fixed_lights.rs"]
mod scene_fixed_lights;
#[path = "../../structure/source_size/scene_world_project_io.rs"]
mod scene_world_project_io;
#[path = "../../structure/source_size/scene_world_property_access.rs"]
mod scene_world_property_access;
#[path = "../../structure/source_size/scene_world_render_lights.rs"]
mod scene_world_render_lights;
#[path = "source_size/text_raster_pool.rs"]
mod text_raster_pool;
#[path = "../../structure/source_size/texture_desc_settings.rs"]
mod texture_descriptor_settings;
#[path = "../../structure/source_size/ui_accessibility_extract.rs"]
mod ui_accessibility_extract;
#[path = "../../source_size/ui_component_catalog_editor_showcase.rs"]
mod ui_component_catalog_editor_showcase;
#[path = "../../source_size/ui_component_state_reducer_keyboard_menu.rs"]
mod ui_component_state_reducer_keyboard_menu;
#[path = "../../source_size/ui_component_state_reducer_tree_view.rs"]
mod ui_component_state_reducer_tree_view;
#[path = "../../source_size/ui_dispatch_bound_text_model_updates.rs"]
mod ui_dispatch_bound_text_model_updates;
#[path = "../../structure/source_size/ui_dispatch_input_manager.rs"]
mod ui_dispatch_input_manager_tests;
#[path = "source_size/ui_layout_arrange.rs"]
mod ui_layout_arrange;
#[path = "../../structure/source_size/ui_secure_text_keyboard_policy.rs"]
mod ui_secure_text_keyboard_policy;
#[path = "../../source_size/ui_surface_default_interactions.rs"]
mod ui_surface_default_interactions;
#[path = "../../structure/source_size/ui_surface_event_routing.rs"]
mod ui_surface_event_routing;
#[path = "../../structure/source_size/ui_surface_property_mutation.rs"]
mod ui_surface_property_mutation;
#[path = "../../structure/source_size/ui_surface_render_feedback.rs"]
mod ui_surface_render_feedback;
#[path = "../../structure/source_size/ui_surface_table_columns.rs"]
mod ui_surface_table_columns;
#[path = "source_size/ui_template_document.rs"]
mod ui_template_document;
#[path = "../../structure/source_size/ui_template_style_apply.rs"]
mod ui_template_style_apply;
#[path = "source_size/ui_text_layout.rs"]
mod ui_text_layout;
#[path = "source_size/ui_v2_style.rs"]
mod ui_v2_style;

fn read_runtime_src(relative: &str) -> String {
    normalize_line_endings(
        std::fs::read_to_string(runtime_src_path(relative))
            .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}")),
    )
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
