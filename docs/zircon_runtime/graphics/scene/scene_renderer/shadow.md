---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_shadow_captures_directional_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_wide.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_shadow_captures_directional_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_wide.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
plan_sources:
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ShadowUtils.cs
  - dev/bevy/crates/bevy_light/src/cascade.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs::render_shadow_atlas_allocates_tiers_descending
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs::render_shadow_atlas_global_downgrade_fits_pressure
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs::render_shadow_atlas_evicts_lowest_priority_on_pressure
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs::render_shadow_atlas_hysteresis_prevents_flapping
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs::render_shadow_atlas_preempts_after_confirmed_priority_margin
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator/tests.rs::render_shadow_atlas_scale_bias_matches_slice_transform
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_splits_blend_log_linear
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_ranges_are_monotonic_and_have_fade_bands
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_snapping_quantizes_origin
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_view_projection_is_stable_under_half_texel_motion
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_bounds_follow_camera_slice_depth
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_layout_matches_plan_05_std430_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_disabled_has_no_valid_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_from_allocation_writes_atlas_slice_and_flags
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_encodes_pcf_quality_in_flags
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_globals_layout_and_atlas_params_are_stable
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_resource_config_normalizes_zero_values
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_resource_config_uses_plan_05_defaults
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_compare_function_matches_forward_depth_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_resource_config_downgrades_to_capability_limit
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_upload_report_describes_cleared_tail
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs::tests::render_shadow_atlas_group1_bindings_avoid_legacy_shadow_and_light_grid_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs::tests::render_shadow_atlas_group1_layout_entries_match_plan_05_resource_types
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_receives_shadow_atlas_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_encodes_shading_model_and_receive_shadow_flag_into_gbuffer_material_alpha
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_receives_shadow_atlas_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_decodes_shading_model_and_receive_shadow_flag_from_gbuffer_material_alpha
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs::deferred_material_gbuffer_shaders_encode_and_decode_material_channels
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs::render_product_directional_shadow_atlas_forward_deferred_darkening_parity
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs::render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs::tests::builtin_pbr_shader_receives_shadow_atlas_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_frame_plan_assigns_first_directional_cascade_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_frame_plan_caps_directional_cascade_tier_to_atlas_row
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_frame_plan_builds_distinct_directional_cascade_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_frame_plan_assigns_point_light_contiguous_face_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_frame_plan_assigns_spot_light_slot_view_key
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_frame_plan_encodes_per_light_pcf_quality
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_feature_registers_hzb_ray_march_pass
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_graph_pass_is_absent_when_plugin_feature_is_disabled
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_executor_accepts_declared_pass_contract
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_executor_requires_gpu_after_contract_validation
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_executor_rejects_resource_contract_drift
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_shader_declares_expected_compute_bindings
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs::contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/wgpu_product_tests.rs::contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs::tests::pluginized_rendering_feature_names_drive_runtime_post_process_flags
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::contact_shadow_runtime_flag_is_encoded_separately_from_ssao
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_contact_shadow_occlusion_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan/tests.rs::render_shadow_light_slot_assignments_patch_packed_light_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs::runtime_15_shadow_plan_view_projection_is_child_owner
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs::runtime_15_shadow_atlas_plan_tests_are_child_owners
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_shadow_captures_directional_tests.rs::runtime_15_render_product_shadow_captures_directional_tests_are_child_owner
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs::tests::shadow_atlas_view_filter_keeps_only_visible_source_entities
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs::tests::shadow_atlas_binds_forward_shadow_receiver_layout_slot
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs::tests::shadow_map_scene_bind_group_matches_environment_scene_layout (2026-07-05 direct binary passed 1/1; log docs/tests/runtime/render/plan08_shadow_scene_bind_group_environment_layout_direct_binary_20260705.out.log)
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct/tests.rs::visibility_context_builds_shadow_views_for_atlas_light_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs::tests::mesh_visibility_states_preserve_shadow_only_casters
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_batch_ref_emits_gpu_scene_instance_command
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::render_product_csm_directional
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::render_product_multi_spot_shadows
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs::render_product_directional_shadow_atlas_capture_records_receiver_path
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs::render_product_directional_shadow_atlas_darkens_receiver_capture
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs::render_product_csm_directional_remains_stable_under_subtexel_camera_shift
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs::render_product_multi_spot_shadow_atlas_darkens_receivers_capture
  - zircon_runtime/src/graphics/tests/render_product_shadow_wide.rs::render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_forward_plus_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_deferred_preserves_shadow_atlas_required_external_texture_binding
doc_type: module-detail
---

# Scene Renderer Shadow Support

This module is the Plan 05 LS-M3/LS-M4 shadow foundation. The compiled graph declares a `shadow-atlas` pass using the `shadow.atlas` executor and writes the persistent `shadow-atlas` depth resource. The atlas, cascade, slot ABI, WGPU resource owner, group1 bindings, multi-light sampling WGSL, slot-depth replay path, and per-light PCF quality contract are in place; the legacy single `SHADOW_MAP` receiver/resource has been removed.

## Current Boundary

- `shadow_map_renderer.rs` owns the atlas-slot replay path and per-slot scene uniform, but no longer owns fixed Shadow WGPU pipelines or inline shadow WGSL. Shadow replay asks `MeshPipelineCache::ensure_shadow_pipeline_for_variant(...)` for the command's `MeshPipelineVariantId`; `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs` assembles `mesh_pipeline_shadow_template_source_for_geometry(...)`, feeds the shared disk/source-hash cache, and `graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs` creates the depth-only or alpha-mask WGPU pipeline from `graphics/shader/wgsl/zr_template_shadow.wgsl` / `graphics/shader/wgsl/zr_template_shadow_alpha.wgsl`. The renderer still records atlas slot depth passes by updating the scene uniform per planned slot, setting atlas viewport/scissor to the slot rect, and replaying the shadow command stream into the shared atlas view. Atlas slot passes carry Plan 04 `VisibilityViewKey` values for directional cascades, point faces, and spot shadows; when a matching view exists on the frame, replay skips shadow commands whose source entity is not visible in that shadow view. The old single-map receiver uniform, direct single-map recording path, and renderer-local inline shadow WGSL path have been deleted.
- The atlas depth replay path uses the full mesh pipeline layout even when the pass writes only depth. It binds scene group 0, a fallback forward-shadow-receiver group 1, the command's standard material group 2, then GPUScene/geometry state before issuing indirect draws. This keeps opaque and alpha-mask shadow depth variants compatible with the same material/receiver layout used by forward, deferred, and prepass mesh pipelines instead of creating a shadow-only compatibility layout.
- Scene group 0 for shadow replay follows the shared five-slot environment layout from `scene_bind_group_layout_entries()`: scene uniform, source environment cube, filtering sampler, BRDF LUT, and specular PMREM cube. `ShadowMapRenderer` owns retained 1x1 fallback cube/LUT/PMREM textures and a sampler for this depth-only path so `zircon-shadow-map-scene-bind-group` cannot drift from the runtime scene renderer or WGPU prewarm pipeline validation. Status: `render_plan08_material_filter_scene_environment_5slot_shadow_prewarm_direct_binary_passed_ui_layout_open`.
- `shadow/atlas/allocator.rs` owns frame-local atlas slot planning only. It does not create WGPU textures or record shadow passes yet.
- `shadow/atlas/bindings.rs` fixes the final group1 atlas binding ABI as 8/9/10/11. Forward and deferred lighting bind groups now include only the atlas receiver entries plus light-grid buffers; the old single-shadow receiver bindings are gone.
- `shadow/atlas/resources.rs` owns the persistent WGPU atlas texture/view, comparison sampler, `shadow_slots` storage buffer, and `shadow_globals` uniform buffer. `SceneRendererCore` creates it and uploads the current `ShadowFramePlan` payload each render.
- `shadow/cascade.rs` owns CSM split/fade/snapping math and camera frustum slice bounds. The slice-bounds helper is crate-reexported through `graphics::scene` so `ShadowFramePlan` and Plan 04 visibility build the same directional cascade coverage without exposing the private shadow module tree.
- `shadow/plan.rs` bridges the full viewport frame to atlas allocation, `GpuShadowSlot`/`GpuShadowGlobals` upload payloads, `ShadowAtlasSlotPass` depth-write descriptors, and `GpuLightData.shadow_slot_layer` patching. It tags atlas slot passes with `VisibilityViewKey::ShadowCascade`, `VisibilityViewKey::ShadowPointFace`, or `VisibilityViewKey::ShadowSpot`, but delegates view-projection matrix construction to the child owner below.
- `shadow/view_projection.rs` owns directional cascade, spot, and point-face view-projection matrices plus direction fallback, stable up-vector, finite vector, and far-plane sanitizing. Plan 05/09 shadow view-projection owner split (`render_plan05_09_shadow_view_projection_owner_split_static_passed`) added this boundary and `runtime_15_shadow_plan_view_projection_is_child_owner` to keep these helpers from flowing back into `shadow/plan.rs`.
- Plan 05/F12 shadow cleanup (`render_plan05_shadow_dead_code_suppression_cleanup_static_passed_cargo_deferred_active_lanes`) keeps the shadow root and atlas root declaration-only: `shadow/mod.rs` has no module-level dead-code suppression, `shadow/atlas/mod.rs` exports only live allocator/resource/binding contracts, and `ShadowFramePlan` no longer stores or exposes the unused atlas allocation report after deriving the live slot/pass/light assignment payload.
- Plan 05 shadow atlas/plan test owner split (`render_plan05_shadow_atlas_plan_test_owner_split_static_passed_cargo_deferred_active_compile_lane`) keeps `shadow/atlas/allocator.rs` and `shadow/plan.rs` as production owners and moves their allocation-pressure, cascade, point, spot, PCF, and light-slot writeback coverage into `shadow/atlas/allocator/tests.rs` and `shadow/plan/tests.rs`. Guard `runtime_15_shadow_atlas_plan_tests_are_child_owners` keeps the moved tests from returning to the production owners and enforces the four-file 800-line budget.
- `shadow/slot.rs` owns the GPU POD layout for shadow slots/globals. Buffer ownership exists in `ShadowAtlasResources`, and the forward/deferred group1 bindings now expose those buffers to fragment shaders.
- `shadow/shaders/zr_shadow.wgsl` owns the shader-side atlas sampling helper. It reads `GpuLightData.shadow_slot_layer`, chooses directional cascades or point faces, projects through `ZrShadowSlot.view_proj`, and selects Low/Medium/High comparison-sampler PCF kernels from the slot quality flags. High keeps nine samples but uses a wider 8-texel radius so product captures can distinguish it from Low on receiver edges.
- `PostProcessGraphResourceNames::SHADOW_ATLAS` names the graph-visible external atlas resource. The built-in `shadow-atlas` pass writes it as a required external texture, and forward mesh/deferred lighting/deferred transparent mesh declare required texture reads so graph ordering keeps atlas depth production before atlas sampling. `PostProcessGraphResourceNames::SHADOW_MAP` is no longer part of the runtime graph contract.
- `RenderShadowExecutionReport` reports graph execution, atlas write/read, receiver availability, caster draw counts, and `shadowed_light_count`. The latter is submit-side counted from the atlas-supported shadow-casting light families(directional, point, and spot), so multi-spot scenes are no longer reported as only the directional ready count.
- `PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION` names the optional screen-space contact shadow output. The `rendering.contact_shadow` plugin owns the HZB-driven `contact-shadow` pass and its WGPU compute executor/shader, so atlas shadows remain built-in while short-distance screen-space shadowing stays opt-in. The executor reads `scene-depth`, `gbuffer-normal`, and `hzb-furthest`, writes the Rgba8Unorm visibility output, resolves pass-declared texture views through `RenderPassGpuExecutionContext::require_texture_view(...)`, and records its compute dispatch through the public plugin-facing GPU context method. The built-in `post.stack` pass declares a read from that texture and the post-process shader samples binding 27 under `contact_shadow_enabled`; feature-off or missing-resource paths bind a white fallback, so no visual multiplier is applied. `contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region` now proves that path darkens a real Forward+ product capture against a baseline pipeline, and `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions` broadens it to a 192x128 wide receiver with left/center/right contact windows while guarding an open receiver region against blanket full-frame deltas.
- `core/framework/render/light/shadow_settings.rs` remains the framework-facing authoring contract for `casts_shadow`, bias, strength, resolution preference, and `ShadowPcfQuality`.

## Atlas Allocator

`ShadowAtlasConfig::default()` follows Plan 05: a 4096x4096 atlas with the top 1024 pixels reserved for directional CSM. Tests use smaller square atlases through `new_square`.

Allocation input is `ShadowSlotRequest`:

- `ShadowSlotKey { light_id, face_index }` identifies one shadow slice. A point light will use six keys in the later GPU pass slice.
- `requested_tier` and `minimum_tier` use `ShadowResolutionTier` from the framework contract.
- `priority` is a finite non-negative score used after global downgrade.

The allocator first applies the URP-style total-area estimate: if the requested area exceeds the available atlas area, all requests are downshifted by powers of two before packing, while respecting each request's minimum tier. Packing then uses a deterministic free-rectangle shelf variant ordered by y/x, so allocations are stable and non-overlapping.

Hysteresis is local to `ShadowAtlasAllocator`:

- A previous allocation is reused for the same key while the request stays present, the tier is unchanged, and the rect remains free.
- Slot retention uses the Plan 05 constants: 8 frames retention, 4 consecutive contention frames, and a 25% priority margin before confirmed preemption.
- The frame result records both `allocations` and `rejected` slots so later graph diagnostics can expose downgrade/pressure behavior.

`ShadowSlotAllocation::atlas_scale_bias()` mirrors URP `ApplySliceTransform`: `[scale_x, scale_y, offset_x, offset_y]` in normalized atlas UV space.

## WGPU Atlas Resources

`bindings.rs` reserves the final group1 range for atlas sampling without conflicting with light-grid bindings:

| binding | resource |
|---------|----------|
| 8 | `shadow_atlas` depth texture |
| 9 | comparison sampler |
| 10 | `shadow_slots` read-only storage buffer |
| 11 | `shadow_globals` uniform buffer |

These constants are code-owned and active in the forward/deferred group1 layouts. Bindings 0/1/2 in the forward shadow receiver group and 5/6/7 in deferred lighting are no longer declared by the atlas receiver layouts.

`ShadowAtlasResources` is the persistent resource owner for the graph-declared `shadow-atlas` path. It creates:

- A depth atlas texture using the renderer depth format, with render-attachment and texture-binding usages.
- A default atlas view and comparison sampler. The sampler uses `SHADOW_ATLAS_COMPARE_FUNCTION == GreaterEqual`, matching the forward-depth visibility contract where a receiver is lit when the atlas depth is greater than or equal to the receiver depth.
- A storage buffer sized as `slot_capacity * GPU_SHADOW_SLOT_STRIDE`.
- A uniform buffer initialized with disabled `GpuShadowGlobals`.

`ShadowAtlasResourceConfig::default()` follows Plan 05: 4096x4096 and 256 shadow slots. Construction clamps the atlas to device capability and falls back to 2048x2048 when a device cannot host the requested 4096 dimension. This keeps LS-M3 compatible with lower-limit WGPU adapters while preserving the default design for capable devices.

`upload_frame()` writes packed `GpuShadowSlot` data and `GpuShadowGlobals`. When a later frame uploads fewer slots than the previous frame, the stale tail is explicitly overwritten with disabled slots so shader-visible storage cannot retain old valid flags. `SceneRendererCore::render_compiled_scene()` and the legacy `render_scene()` path build a `ShadowFramePlan`, upload its slots/globals, pass its light-slot assignment table into GPUScene light packing, and make the uploaded atlas resources available to forward/deferred graph execution. The compiled graph import step binds `SHADOW_ATLAS` to this persistent atlas view, and the required external texture binding means materialization validation fails if that atlas view is missing before executor dispatch.

## Slot ABI

`GpuShadowSlot` is the CPU-side `ZrShadowSlot` ABI from Plan 05:

- `view_proj` at offset 0, 64 bytes.
- `atlas_scale_bias` at offset 64.
- `params` at offset 80, with `x = depth_bias`, `y = normal_bias`, `z = slot_texel_size`, and `w = bitcast flags`.

`GpuShadowSlot::from_allocation()` converts a `ShadowSlotAllocation` plus a light-space matrix into the atlas slice transform and sets the valid flag. Slot flags distinguish directional cascades, spot slices, and point-light cube faces; bits 8..9 encode `ShadowPcfQuality` as Low=1 tap, Medium=5 tap, and High=9 tap. `ShadowPcfQuality::default()` is Low; current product coverage treats High as the wider quality tier by sampling the 9-tap kernel at an 8-texel radius.

`GpuShadowGlobals` is a 48-byte uniform block for cascade far splits, fade lengths, and atlas size/inverse-size. The buffer exists in `ShadowAtlasResources`, and the atlas binding ABI is fixed at group1 bindings 8/9/10/11. Forward/deferred shaders include `zr_shadow.wgsl` and assert the legacy single-shadow receiver symbols are absent.

## Cascades

`CascadeSplitConfig` supports up to `MAX_SHADOW_CASCADES == 4` and computes Plan 05's log/linear blended split distances:

```text
split_i = lerp(linear_i, logarithmic_i, lambda)
```

`compute_cascade_ranges()` turns the split array into per-cascade near/far/fade bands. `cascade_shadow_bounds_from_camera_slice()` derives the eight world-space frustum corners for one camera slice, averages them into a cascade center, and stores a stabilized radius for downstream orthographic coverage. `snap_light_space_center_to_texel()` and `snapped_cascade_view_projection()` provide the texel-grid stabilization used by the directional CSM view construction. The snapping follows the Bevy/UE pattern of quantizing light-space x/y to world-units-per-texel before building the orthographic matrix.

## Frame Plan And Light Writeback

`build_shadow_frame_plan()` is the current LS-M3 bridge between authored lights and GPU light-buffer writeback:

- The first shadow-casting directional light reserves contiguous cascade slots at the start of the shadow slot table, writes `GpuShadowGlobals` split/fade data, and builds texel-snapped orthographic matrices around the current camera frustum slice bounds.
- Point lights request six atlas allocations and only receive a light assignment when all six faces fit; each face writes a 90-degree perspective matrix for the corresponding cube direction.
- Spot lights request one atlas allocation and write a perspective matrix from authored position, direction, cone angle, and range.
- Each slot copies the source light's `pcf_quality` into the GPU flags, so point and spot lights can choose different PCF kernels in the same frame.
- Every emitted GPU slot also emits a `ShadowAtlasSlotPass { slot_index, rect, view_proj, view_key }`. The compiled graph shadow stage passes the frame plan into `RenderPassGpuExecutionContext`; `ShadowMapRenderer::record_atlas_commands_with_attachment_ops()` then writes each slot by setting viewport/scissor to `rect` and replaying the shadow command stream with that slot's `view_proj`. Directional, point-face, and spot slots filter replay through the matching Plan 04 shadow view when present. The filtering remains command-by-command for view-filtered slots so the unfiltered shadow indirect args stream cannot be applied to a narrowed per-slot entity set.
- `ShadowLightSlotAssignments::apply_to_packed_lights()` patches packed `GpuLightData.shadow_slot_layer.x` with the first slot and `shadow_params.w` with the slot count before `GpuScene::write_lights()`.

`SceneRendererCore` owns both `ShadowAtlasAllocator` and `ShadowAtlasResources`. The allocator config is derived from the actual atlas resource size so capability fallback to 2048 keeps atlas rects and GPU resource dimensions aligned. Directional cascade slot tier is capped by the actual atlas width divided by cascade count and by the reserved CSM row height.

This is not yet the final shadow renderer. The plan now produces real `view_proj` matrices and proves slot assignment, buffer upload, light-buffer ABI writeback, group1 binding, forward/deferred atlas sampling entry points, graph-visible `shadow-atlas` ordering, and per-slot atlas depth writes. Directional cascade, point-face, and spot atlas writes are narrowed by Plan 04 shadow-view visibility when matching views exist. Directional cascades now use camera frustum slice bounds for both atlas slot matrices and visibility shadow cameras. `mesh_visibility_states_preserve_shadow_only_casters` now guards the mesh-draw mapping layer so a caster culled from the main view but visible in a shadow view remains eligible for the shadow pass. `render_product_csm_directional` and `render_product_multi_spot_shadows` add product-contract source coverage for CSM slot generation and multi-spot atlas coexistence. `render_product_directional_shadow_atlas_capture_records_receiver_path` adds a real WGPU Forward+ path guard for directional atlas execution, atlas write, receiver read, caster draw, and visible receiver pixels. `render_product_directional_shadow_atlas_darkens_receiver_capture` now proves the same atlas path creates a visible receiver darkening delta under a same-color receive-shadow toggle after the comparison sampler was aligned with the atlas depth contract. `render_product_csm_directional_remains_stable_under_subtexel_camera_shift` proves the same receiver/caster product lane keeps stable darkening statistics under a subtexel camera shift after cancelling ordinary projection movement through a matching unshadowed baseline. `render_product_multi_spot_shadow_atlas_darkens_receivers_capture` extends that product lane to a 3 spot/3 caster scene and its direct WGPU execution now passes, checking `shadowed_light_count`, atlas receiver availability, caster draw count, and full-frame darkened-pixel deltas. `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture` broadens the same product lane to one directional plus three spot shadow-casting lights in one frame, checking the shared shadow atlas and clustered light-grid executors plus left/center/right receiver darkening regions. `render_product_directional_shadow_atlas_forward_deferred_darkening_parity` now proves the same directional atlas receiver darkening contract in Forward+ and Deferred after Deferred material alpha began preserving the receive-shadow flag beside the shading model. `render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture` proves Low/High PCF quality changes are visible in a real WGPU spot receiver-edge capture. `contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region` proves the opt-in plugin contact-shadow path executes and darkens a real Forward+ product capture, and `contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions` broadens that plugin path to three localized contact regions plus an open-region false-positive guard. Remaining precision work is RenderDoc evidence, root wider sweeps, and any real-scene caster/contact expansion found by broader validation.

## Next Integration

The current LS-M3/LS-M4 real-capture lane has directional atlas path, directional receiver darkening, CSM subtexel camera-shift stability, multi-spot receiver darkening, mixed directional+spot wider receiver darkening, Forward+/Deferred directional shadow parity, and spot PCF quality edge guards. The next slices still need to broaden shadow behavior:

- Directional cascades occupy the reserved atlas row and write slots `0..cascade_count`; the product guards prove this path reaches a receiver frame and can visibly darken a receiver sample.
- Spot shadows allocate one slot; point shadows allocate six cube-face slots.
- `GpuLightData.shadow_slot_layer` writeback and shader sampling now have atlas depth input, directional receive-shadow visual delta, Deferred receive-shadow G-buffer flag preservation, and a passing multi-spot receive-shadow WGPU product guard after the VG debug-snapshot fixtures were synced and the lib-test binary was produced with a longer `--no-run` window.
- `GpuShadowSlot`, `GpuShadowGlobals`, and `ShadowAtlasSlotPass` are populated from real allocation/cascade data; directional cascade, point-face, and spot slots consume Plan 04 shadow-view visibility, and directional cascades use camera-frustum slice bounds. Source coverage now preserves shadow-only casters at mesh-draw visibility mapping, while the remaining CSM-adjacent risk is any real-scene caster expansion or receiver-slice issue found by broader scenes.
- Pass/executor naming has converged to `shadow-atlas`/`shadow.atlas`; the graph/resource hard cut is complete. Source contracts now cover multi-spot coexistence and CSM slot generation, and real-capture coverage now includes directional and multi-spot receiver darkening, mixed directional+spot wider receiver darkening, CSM subtexel stability, Forward+/Deferred directional shadow parity, PCF quality edge differences, and plugin contact-shadow darkening across single and wider multi-region scenes. Validation still needs RenderDoc evidence and root wider sweeps.

## Validation State

The 2026-07-04 Plan 08 broad `render_product_` current-source repair fixed the shadow-atlas WGPU validation failures that reported missing bind groups for `zircon-shadow-depth-mesh-pipeline`. The support-layer fix stays in `shadow_map_renderer.rs`: per-slot atlas passes create a fallback forward-shadow-receiver bind group, bind it at group 1, bind the standard-material group at group 2 through `MeshDrawCommandReplayer::bind_standard_material_if_needed(...)`, and then bind GPUScene/geometry before shadow depth indirect draws. Source guard `shadow_atlas_binds_forward_shadow_receiver_layout_slot` passed, the focused spot PCF capture passed, `graphics::tests::render_product_shadow_captures` passed 7/7, `graphics::tests::render_product_shadow_wide` passed 1/1, and the final broad `render_product_` direct generated-binary rerun passed 203 automatic tests with 0 failures and 6 ignored. RenderDoc/product capture, default-feature broad reruns, and workspace/full CI remain separate gates.

The 2026-06-24 Plan 08 Shadow pipeline template source cache cutover removed the old renderer-local shadow source/inline WGSL path and moved ShadowDepth/ShadowDepthAlphaMask WGPU pipeline creation to `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs` plus `graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs`; the pass sources are `graphics/shader/wgsl/zr_template_shadow.wgsl` and `graphics/shader/wgsl/zr_template_shadow_alpha.wgsl`. `ShadowMapRenderer` now resolves `command.pipeline_variant_id` through `MeshPipelineCache`, while shadow command producers and the compiled graph Shadow stage pass real variant IDs and mesh pipeline context. Guard `runtime_15_render_shader_template_assembly_is_folder_backed` locks the template source owner, source-hash shader key, static vertex layout, depth-bias contract, no stale shadow shader module, `shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash`, `shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias`, and status anchor `render_plan08_shadow_pipeline_template_source_cache_static_passed_cargo_check_test_compile_wgpu_deferred`. Current evidence is scoped rustfmt/static scans, line-count scan, docs-anchor scan, touched-file whitespace scan, conflict-marker scan, scoped diff-check, scoped `zircon_runtime --lib` cargo check passing with existing warnings, and focused structure-guard no-run compile passing with existing warnings after explicit folder-backed module paths were restored. WGPU device pipeline validation and RenderDoc/product acceptance are not counted as passed.

The 2026-06-24 Plan 05 shadow atlas/plan test owner split moved LS-M3 allocator and frame-plan unit coverage into child owners without changing production behavior. `shadow/atlas/allocator.rs` is now 683 lines, `shadow/atlas/allocator/tests.rs` is 215 lines, `shadow/plan.rs` is 506 lines, and `shadow/plan/tests.rs` is 360 lines. Guard `runtime_15_shadow_atlas_plan_tests_are_child_owners` locks the moved-test ownership, four-file line budget, and Plan 05/render index/structure/review/shadow docs anchors. Current evidence is scoped rustfmt/static scans, line-count scan, docs-anchor scan, touched-file whitespace scan, and scoped diff-check; Cargo/WGPU/RenderDoc remain deferred because active compile lanes were present.

The 2026-06-24 Render product shadow captures directional test owner split keeps `graphics/tests/render_product_shadow_captures.rs` as the spot PCF, multi-spot receiver, and shared fixture parent and moves directional atlas path/darkening, CSM subtexel stability, Forward+/Deferred directional parity, and directional capture helpers into `graphics/tests/render_product_shadow_captures/directional.rs`. Guard `runtime_15_render_product_shadow_captures_directional_tests_are_child_owner` locks the moved-test/helper ownership, parent/child 800-line budget, and Plan 05/render index/structure/review/render-product/shadow docs anchors. Status anchor: `render_plan05_shadow_capture_directional_tests_owner_split_static_passed_cargo_deferred_implementation_cadence`. Current evidence is scoped rustfmt/static scans, line-count scan, docs-anchor scan, stale-path scan, touched-file whitespace scan, and scoped diff-check; Cargo/WGPU/RenderDoc remain deferred by milestone implementation cadence.

The 2026-06-24 VisibilityContext construct tests owner split keeps Plan 04 shadow-view source coverage outside the production constructor: `graphics/visibility/context/from_extract_with_history/construct.rs` now only mounts tests, while `graphics/visibility/context/from_extract_with_history/construct/tests.rs` owns `visibility_context_builds_shadow_views_for_atlas_light_slots` and related visibility fixtures. Guard `runtime_15_visibility_context_construct_tests_are_child_owner` locks that boundary and the status anchor `render_plan04_visibility_context_construct_tests_owner_split_static_passed_cargo_deferred_active_compile_lane`; current evidence is scoped rustfmt/static/line-count/docs-anchor/whitespace/diff-check only, with Cargo/WGPU/RenderDoc deferred because active compile lanes were present.

The 2026-06-13 directional cascade slice follow-up passed the core-min library check in `E:\cargo-targets\zircon-render-vc3-compact-replay-coremin`, compiled the shared lib-test target with `--no-run`, then ran `cargo test ... render_shadow_ -- --nocapture` successfully for 27 filtered shadow tests. The same target also ran `visibility_context_builds_shadow_views_for_atlas_light_slots` successfully as one filtered visibility test. During debugging, the pre-existing half-texel snapping test data was corrected because the old negative Y sample crossed a texel floor boundary; implementation behavior was unchanged.

The 2026-06-13 pass/executor naming slice converged the graph contract to `shadow-atlas` / `shadow.atlas` while retaining the legacy `SHADOW_MAP` resource as the receiver compatibility surface. Validation passed `cargo fmt --all -- --check`, scoped `git diff --check`, and the core-min library check in `E:\cargo-targets\zircon-render-vc3-compact-replay-coremin` with existing warnings. The focused `shadow_atlas` Cargo filter passed 16 tests and ignored the WGPU submit stats test because the current HZB occlusion path exceeds an 8-storage-buffer adapter limit on this machine. Direct execution of the built core-min lib-test binary passed all 4 `render_product_shadows` contract tests.

The 2026-06-14 receiver hard cut removed the legacy `SHADOW_MAP` graph resource, forward/deferred single-map receiver bindings, `ShadowReceiverUniform` shader structs, Rust receiver uniform buffers, and old single-map depth recording helper. The shadow graph now writes only external `SHADOW_ATLAS`, and render stats count `shadow_atlas_write_count`. Validation passed `cargo fmt --all`, `cargo fmt --all -- --check`, scoped `git diff --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-shadow-atlas-cutover-coremin --message-format short --color never` with existing warnings. A focused Cargo test run for `shadow_atlas` could not start under `--locked` because the current `Cargo.lock` does not satisfy test-target resolution; the lock file was not modified.

The 2026-06-14 caster/receiver source-guard slice added `mesh_visibility_states_preserve_shadow_only_casters`, proving that mesh draw visibility states keep a shadow-view-visible caster even when the main view culled it. Validation passed `cargo fmt --all`, `cargo fmt --all -- --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-caster-receiver-coremin --message-format short --color never` with existing warnings. A focused Cargo test run for the new test timed out after 904 seconds while building/running the shared lib-test target; target-dir cargo/rustc processes were stopped, and `Cargo.lock` remained unchanged.

The 2026-06-14 product shadow contract slice added `render_product_csm_directional` and `render_product_multi_spot_shadows`, proving the product test suite now has source coverage for directional CSM four-slot generation and at least three simultaneous spot shadow atlas slots without overlap. Validation passed `cargo fmt --all`, `cargo fmt --all -- --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-product-shadow-contracts-coremin --message-format short --color never` with existing warnings. A focused `cargo test --no-run` attempt for `render_product_multi_spot_shadows` timed out after 904 seconds during shared lib-test compilation, so no filtered test result is claimed.

The 2026-06-21 directional shadow-atlas capture slice added `render_product_directional_shadow_atlas_capture_records_receiver_path` and `render_product_directional_shadow_atlas_darkens_receiver_capture`, now owned by `render_product_shadow_captures/directional.rs`. The first test renders a real WGPU Forward+ receiver/caster product frame and asserts `shadow.atlas` executor execution, atlas write count, receiver availability, non-zero caster draw count, and non-zero receiver luma in the captured frame. The second submits same-color receiver scenes with receive-shadows enabled and disabled, then proves the enabled receiver sample is visibly darker. Debugging showed slot projection and raw atlas depth were valid, so the sampler contract was corrected to `SHADOW_ATLAS_COMPARE_FUNCTION == GreaterEqual` and locked by `render_shadow_atlas_compare_function_matches_forward_depth_contract`. Validation passed `cargo test -p zircon_runtime --lib render_shadow_atlas_compare_function_matches_forward_depth_contract --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --quiet -- --test-threads=1 --nocapture`, direct exact reruns of both product tests from the latest warmed binary, and scoped `rustfmt --edition 2021 --check` for `shadow/atlas/resources.rs`, `render_product_shadow_captures.rs`, and `tests/mod.rs`.

The 2026-06-21 multi-spot shadow-atlas guard slice added `render_product_multi_spot_shadow_atlas_darkens_receivers_capture` and `RenderShadowExecutionReport.shadowed_light_count`. The product scene registers three spot lights, three caster meshes, and one receiver, then compares receive-shadows on/off captures by counting darkened pixels and luma/RGB deltas. Submit stats now count atlas-supported shadow-casting directional/point/spot lights instead of reusing directional ready count for the shadow report. Validation passed scoped `rustfmt` and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never`. A fresh `cargo rustc -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --cfg test --emit=metadata` attempt returned -1; the captured log had no Rust error, but it also had no `Finished` line, so it is not counted as passing. The support follow-up synced four direct `RenderMeshSnapshot` fixtures in `virtual_geometry_debug_snapshot_contract.rs` with `stable_instance_key`, `transform_revision`, `mesh_lod`, and `static_state`, after which `cargo check -p zircon_runtime --tests --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never` passed with existing warnings. A longer lib-test `--no-run` window then passed in 18m11s and produced `zircon_runtime-c339c28ec98a5de7.exe`; direct binary execution of `render_product_multi_spot_shadow_atlas_darkens_receivers_capture --nocapture --test-threads=1` passed 1/1 in 8.60s.

The 2026-06-22 mixed shadow-atlas wider guard slice added `render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture` in `render_product_shadow_wide.rs`. The guard keeps `render_product_shadow_captures.rs` from growing further, submits a real Forward+ frame with one directional and three spot shadow-casting lights, and compares receive-shadows on/off captures across the whole frame plus left/center/right receiver regions. It also asserts `shadow.atlas`, `lighting.light-grid`, directional/spot readiness, `RenderShadowExecutionReport.shadowed_light_count == 4`, receiver availability, and caster draw count. Validation passed `cargo test -p zircon_runtime --lib render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture --locked --jobs 1 --target-dir target\codex-shadow-wide-0622 --message-format short --color never -- --test-threads=1 --nocapture` with 1/1 in 5.21s after a 28m46s first default-feature lib-test build and the existing warning set.

The 2026-06-21 CSM subtexel stability slice added `render_product_csm_directional_remains_stable_under_subtexel_camera_shift`, now owned by `render_product_shadow_captures/directional.rs`. It renders the same directional receiver/caster product scene in baseline and x=0.006 shifted camera positions, captures shadowed and unshadowed variants for each camera, then compares darkened-pixel count and luma delta between the two camera positions after the matching unshadowed baseline cancels ordinary projection movement. Validation passed `cargo test -p zircon_runtime --lib render_product_csm_directional_remains_stable_under_subtexel_camera_shift --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture` with 1/1 in 3.50s after a 6m34s compile/run window and the existing warning set.

The 2026-06-21 forward/deferred shadow parity slice added `render_product_directional_shadow_atlas_forward_deferred_darkening_parity`, now owned by `render_product_shadow_captures/directional.rs`. The guard renders the same directional receiver/caster scene through Forward+ and Deferred pipelines, compares shadowed/unshadowed capture deltas, and requires the expected `mesh.opaque` versus `lighting.deferred` executor paths. Its first red run showed Deferred had `pixels=0` / `sum=0.00` darkening because Deferred material alpha carried only the shading-model id and lost the receiver `receive_shadows` policy. `deferred_geometry.wgsl` now encodes low 7 bits as shading model plus high bit as receive-shadow flag, while `deferred_lighting.wgsl` decodes that flag before calling `zr_gpu_light_shadow_visibility(...)`. Validation passed the exact parity filter 1/1, `deferred_geometry_shader` 7/7, `deferred_lighting_shader` 6/6, and `deferred_material_gbuffer_shaders_encode_and_decode_material_channels` 1/1 in `target\codex-runtime-shadow-spot-0621` with the existing warning set.

The 2026-06-14 LS-M4 PCF quality source-contract slice added `ShadowPcfQuality`, encoded per-light PCF quality into `GpuShadowSlot.params.w` flags, and switched `zr_shadow.wgsl` from a fixed 3x3 PCF kernel to slot-selected 1/5/9 tap kernels. Validation passed `cargo fmt --all -- --check`, scoped `git diff --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never` with existing warnings. `cargo test -p zircon_runtime --lib pcf_quality --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never --no-run` timed out after 904 seconds while compiling/linking the shared lib-test target; no filtered test result is claimed. The target-dir cargo/rustc leftovers were stopped, and a source-contract scan confirmed the key enum, flag, frame-plan, WGSL, and shader-source symbols are present.

The 2026-06-21 LS-M4 PCF quality capture slice added `render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture`. The guard submits Low, High, and unshadowed baseline frames for a spot receiver/caster edge scene, then requires receiver darkening in both PCF modes and an RGB capture delta between Low and High. The initial directional scene produced identical captures; a temporary diagnostic proved High quality reached `zr_shadow.wgsl`, so the accepted path uses the spot edge scene plus an 8-texel High kernel radius. Validation passed the exact PCF Cargo filter 1/1 and the `shadow_atlas_resources` source-contract filter 3/3 in `target\codex-runtime-shadow-spot-0621` with the existing warning set.

The 2026-06-14 LS-M4 contact shadow descriptor slice added the
`rendering.contact_shadow` optional plugin feature, its runtime/editor crates,
and the `contact-shadow` async compute graph pass descriptor. The pass reads
`scene-depth`, `gbuffer-normal`, and Plan 04 `hzb-furthest`, then writes
`contact-shadow-occlusion`. Scoped rustfmt and diff checks passed, plugin
metadata confirmed both contact-shadow packages, and a 16-symbol source-contract
scan covered the pass, manifest, catalog, graph insertion, and docs contracts.
Locked runtime/plugin cargo checks were blocked before compilation because the
current root/plugin lock files need refresh under `--locked`/`--frozen`; no lock
file was modified in this slice.

The 2026-06-14 LS-M4 contact shadow executor slice replaced the placeholder
executor with a plugin-owned WGPU compute executor and `contact_shadow.wgsl`.
The executor caches its compute pipeline, binds the graph depth/normal/HZB/output
resources directly, dispatches 8x8 workgroups, and records the dispatch plus the
`contact-shadow-occlusion` storage write through
`RenderPassGpuExecutionContext::record_compute_dispatch(...)`. Scoped rustfmt,
scoped diff, and a 15-symbol source-contract scan passed. Locked cargo checks
for root runtime and the contact-shadow plugin still stopped before compilation
because both lock files need refresh; no lock file was modified while other Cargo
tasks were active.

The 2026-06-14 LS-M4 contact shadow post-process consumption slice made
`post.stack` declare a read from `contact-shadow-occlusion`, added the runtime
flag and `PostProcessParams::lighting_flags.x`, expanded the post-process bind
group with binding 27, and multiplied the sampled contact visibility into final
color independently from SSAO. Scoped rustfmt, scoped diff, a 12-symbol
source-contract scan, and a 4-call-site SSR fallback scan passed. Locked Cargo
validation was retried for root runtime and the contact-shadow plugin package,
but both commands stopped before compilation because the corresponding lock files
need refresh under `--locked`; no lock file was modified in this slice.

The 2026-06-21 LS-M4 contact shadow product-capture slice added
`contact_shadow_wgpu_product_capture_darkens_screen_space_contact_region`. The
guard submits matching Forward+ receiver/blocker frames with and without
`rendering.contact_shadow`, checks the effective feature list, `contact-shadow`
pass, `lighting.contact-shadow` executor id, compute dispatch/workload, and zero
graph coverage gaps, then compares final captures for measurable contact-region
darkening. The plugin executor now resolves its pass-declared depth, normal, HZB,
and output texture views through `RenderPassGpuExecutionContext::require_texture_view(...)`,
so plugin WGPU resource access stays behind the renderer-owned resolver boundary.
Validation passed the exact contact-shadow Cargo filter 1/1 and the full
`zircon_plugin_rendering_contact_shadow_runtime --lib` suite 7/7 under
`--locked` in `..\target\codex-plugin-contact-shadow-0621`; `zircon_plugins/Cargo.lock`
was synchronized for the plugin workspace.

The 2026-06-22 wider contact-shadow product guard slice added
`contact_shadow_wgpu_product_capture_darkens_multiple_screen_space_contact_regions`
in `wgpu_product_tests.rs`. The guard keeps the change in the existing
contact-shadow product-test owner(531 lines, under the structure-convention split
threshold), submits plugin-enabled/baseline Forward+ frames for a 192x128 wide
receiver with three blocker groups, requires whole-frame plus left/center/right
contact-window darkening, and checks an open receiver window to prevent accepting
a blanket color delta. It also keeps the feature/pass/executor/dispatch/coverage
assertions. Validation passed exact filter 1/1 and full runtime lib 8/8 using
`..\target\codex-plugin-contact-shadow-0621`; existing warnings only. The slice
was checked against `engine-code-structure-convention.md` and
`engine-code-review-findings-2026-06.md` and adds no production String-error,
dead-code, FFI, or builder/API debt.
