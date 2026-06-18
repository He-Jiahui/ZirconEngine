---
related_code:
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_relevance_entry.rs
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/graphics/visibility/culling/is_mesh_visible.rs
  - zircon_runtime/src/graphics/visibility/culling/mesh_bounds.rs
  - zircon_runtime/src/graphics/visibility/occlusion/mod.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/visibility_static_index.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/default.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/new_compile_options.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/phase_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/zr_hzb.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/hzb_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/constants/hzb.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/hzb.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_buffer_bundle/hzb_params_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/hzb_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/hzb_source_texture_view.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_frame_visibility.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/render_framework_visibility_submit.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/relevance.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_relevance_entry.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/graphics/visibility/culling/is_mesh_visible.rs
  - zircon_runtime/src/graphics/visibility/culling/mod.rs
  - zircon_runtime/src/graphics/visibility/occlusion/mod.rs
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/visibility_static_index.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/default.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/new_compile_options.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/phase_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/zr_hzb.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/hzb_build.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/hzb_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/constants/hzb.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/hzb.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_buffer_bundle/hzb_params_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/hzb_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/hzb_source_texture_view.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_frame_visibility.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/depth_prepass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/opaque_base.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
plan_sources:
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/index.md
  - user: 2026-06-17 bind HZB executor-owned external buffers for render plan 01
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
tests:
  - zircon_runtime/src/core/framework/render/relevance.rs::tests::primitive_relevance_tracks_material_layer_and_motion_policy
  - zircon_runtime/src/core/framework/render/relevance.rs::tests::primitive_relevance_keeps_shadow_eligibility_separate_from_main_view_layers
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs::tests::parallel_frustum_visibility_matches_serial_order_and_results
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_records_relevance_and_filters_main_view_layers
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_builds_shadow_view_independent_from_main_layers
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_builds_shadow_views_for_atlas_light_slots
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_builds_custom_target_view_from_camera_descriptors
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs::tests::frame_submission_context_exposes_view_visibility_by_key
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::update_visibility_stats_sums_per_view_culling_counts
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::processors_keep_shadow_candidate_when_main_view_layer_filters_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs::tests::shadow_processor_respects_shadow_view_visibility
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs::tests::mesh_visibility_states_preserve_shadow_only_casters
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_visibility_stats
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_builder_sizes_odd_viewport_to_half_power_of_two_chain
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_builder_keeps_one_pixel_viewports_valid
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_builder_reduce_passes_cover_tail_mips
  - zircon_runtime/src/graphics/visibility/occlusion/hzb_builder.rs::tests::hzb_build_plan_reports_each_mip_extent
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/hzb_pipeline.rs::tests::hzb_shader_declares_reduce_entry_and_storage_target
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs::tests::hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs::tests::hzb_occlusion_culler_shader_declares_expected_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/phase_dispatch.rs::tests::hzb_occlusion_dispatch_groups_cover_indirect_args
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/phase_dispatch.rs::tests::hzb_occlusion_dispatch_groups_sum_phase_local_workloads
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/phase_dispatch.rs::tests::hzb_occlusion_dispatch_summary_saturates_phase_and_group_counts
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs::tests::hzb_occlusion_uploads_phase_params_in_encoder_order
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs::tests::hzb_occlusion_culler_clears_compaction_outputs_before_culling_dispatch
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs::tests::hzb_external_fallback_buffers_satisfy_materialization_report
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs::tests::hzb_occlusion_cull_declares_execution_owned_external_buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_validation_fails_unbound_required_external_buffer
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs::tests::hzb_occlusion_dispatch_record_reports_compaction_output_writes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_audits_phase_local_indirect_arg_workload_groups
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs::tests::indirect_compaction_metadata_preserves_source_spans_and_prefixes_output_capacity
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs::tests::indirect_compaction_simulation_rewrites_args_to_visible_instance_remap
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction.rs::tests::indirect_compaction_rejects_visible_instance_capacity_overflow
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs::tests::bindable_storage_buffer_size_keeps_zero_capacity_buffers_bindable
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs::tests::mesh_indirect_compaction_resources_reserve_expected_wgpu_usages
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_compaction_resources.rs::tests::mesh_indirect_compaction_resources_clear_outputs_without_rewriting_metadata
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs::tests::mesh_indirect_draw_execution_builds_compaction_plan_from_uploaded_args
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs::tests::visibility_static_index_incremental_update_matches_full_rebuild_queries
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs::tests::visibility_static_index_full_rebuild_strategy_replaces_existing_rows
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_reuses_static_index_without_frame_rebuild
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_rebuilds_static_index_when_previous_index_is_missing
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_uses_static_index_prefilter_above_threshold
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::update_visibility_static_index_stats_records_latest_report
  - zircon_runtime/src/graphics/tests/render_framework_visibility_submit.rs::render_framework_reuses_static_index_and_reports_main_view_prefilter
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs::render_product_hzb_occlusion_wall_scene
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_declares_uber_light_list_frame_resource_for_default_stack
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_declares_uber_light_list_as_external_when_clustered_lighting_is_disabled
  - zircon_runtime/src/graphics/visibility/occlusion/mod.rs::tests::hzb_occlusion_report_preserves_indirect_args_readback_summary
  - zircon_runtime/src/graphics/visibility/occlusion/mod.rs::tests::hzb_occlusion_indirect_args_summary_saturates_totals
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs::tests::mesh_indirect_args_snapshot_counts_zeroed_instance_args
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs::tests::mesh_indirect_draw_execution_sources_readback_from_indirect_args_buffer
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_preserves_hzb_occlusion_cull_report
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_audits_zero_indirect_arg_workload_as_zero_groups
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::update_hzb_occlusion_stats_records_latest_cull_report
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::update_hzb_occlusion_stats_resets_when_no_report
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::hzb_occlusion_culling_requires_storage_buffers_gpu_driven_and_binding_capacity
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs::tests::compile_options_gate_hzb_occlusion_from_backend_capabilities
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compile_options_gate_hzb_occlusion_cull_without_removing_hzb_build
  - zircon_runtime/src/graphics/tests/render_framework_post_process_submit.rs::render_framework_submits_advanced_postprocess_graph_passes
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_hzb_stats
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/core/framework/render/relevance.rs zircon_runtime/src/graphics/visibility/mod.rs zircon_runtime/src/graphics/visibility/declarations/mod.rs zircon_runtime/src/graphics/visibility/declarations/visibility_context.rs zircon_runtime/src/graphics/visibility/declarations/visibility_relevance_entry.rs zircon_runtime/src/graphics/visibility/culling/mod.rs zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/batching_result.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-verify --message-format short --color never
doc_type: module-detail
---

# Visibility Module

## Purpose

`graphics::visibility` is the CPU visibility and culling bridge between render-frame extraction and the mesh/render planning layers. Plan 04 moves this module toward an InitViews-style pipeline: extract renderables, calculate primitive relevance once, run frustum culling, and later feed per-view visibility plus HZB occlusion into mesh command filtering and GPU-driven indirect execution.

This module remains WGPU-free. It consumes `RenderFrameExtract`, `ViewportCameraSnapshot`, `RenderLayerSet`, `GeometryPhaseInput`, and mesh snapshots from `core::framework::render`, then emits `VisibilityContext` data that render submission and planning code can inspect.

## Current VC-M1 Slice

The first VC-M1 code slice adds `PrimitiveRelevance` in `core/framework/render/relevance.rs`. It is a compact bitset describing how a render primitive participates in product phases:

- render-layer match for the active view
- main-view participation
- opaque, alpha-mask, or transparent material class
- depth-prepass eligibility
- shadow-caster eligibility
- deferred geometry eligibility for Core3d opaque-like primitives
- motion-vector candidate eligibility for dynamic opaque-like primitives

The relevance type intentionally separates main-view layer visibility from shadow eligibility. A mesh outside the current camera render layer is not relevant to opaque/alpha/transparent main-view phases or prepass, but an opaque-like mesh can still report shadow-caster eligibility for future independent shadow-view culling.

`VisibilityRelevanceEntry` stores the entity plus its `PrimitiveRelevance`. `VisibilityContext::primitive_relevance` keeps these entries beside batches, history, upload plans, and the authoritative `FrameVisibility`. The legacy flattened `visible_entities`, `culled_entities`, and `visible_batches` fields have been removed; consumers that need the main camera now call `main_view_visible_entities()`, `main_view_culled_entities()`, `main_view_visible_entity_set()`, or `main_view_visible_batches()`, all derived from the frame visibility store instead of copied state.

`FrameVisibility` and `ViewVisibilityContext` now provide the per-view container. The current implementation emits the main camera, directional `ShadowCascade { light, cascade }` views, point-light `ShadowPointFace { light, face }` views, and spot-light `ShadowSpot { light }` views. Shadow-casting directional lights emit four cascade keys to match Plan 05 CSM atlas slots; non-shadow directional lights keep one conservative cascade-0 view for the default-shadow path. Directional cascade cameras now use the same Plan 05 camera frustum slice bounds that drive `ShadowFramePlan`, while point/spot cameras are built from authored light position, direction, range, and cone/face data. It already uses the final shape: a stable frame primitive index space (`entities`, `bounds`, `relevance`) plus view-local visible indices and `ViewCullingStats`. `VisibilityContext` keeps only `frame_visibility` as the per-view authority, so later custom render-target camera slices can extend the view list without reviving main-view-only fields.

## Culling Flow

`collect_batching_result(...)` now builds three lookups from the frame extract:

- mesh snapshots by entity
- geometry phase inputs by entity, to recover the extracted `RenderMaterialAlphaMode`
- visibility renderable entries, falling back to mesh snapshots when the extract does not provide explicit visibility inputs

It then builds a linear candidate array of `{ entity, VisibilityBounds }` and runs `mesh_frustum_visibility(...)` over that array. The helper uses a deterministic serial path for small scenes and, when the render framework supplies its runtime compute task pool, a `core::runtime::tasks::parallel_for(...)` path for larger scenes. The parallel path preserves input order in the collected result, so batch generation remains stable.

Final main-view visibility requires both `PrimitiveRelevance::main_view()` and a positive frustum result. This means camera `RenderLayerSet` filtering now participates in the `FrameVisibility` main-view set and in `VisibilityContext::main_view_visible_batches()`. The main-view layer mask is read through `RenderViewExtract::selected_camera_layers()`, so scene-backed extracts consume the selected `CameraRenderDescriptor` layer ownership instead of any snapshot-side layer field. A layer-mismatched mesh appears in `main_view_culled_entities()`, but its relevance entry remains available for diagnostics and future non-main views.

`FrameVisibility::from_frame_views(...)` converts the same result into index-oriented view data. `ViewVisibilityContext::visible` stores `u32` indices into `FrameVisibility::entities`, not entity ids. Main-view stats count the original primitive input, layer-filtered primitives, frustum-culled primitives, occlusion-culled primitives, and final visible primitive count. Directional shadow views derive orthographic light cameras from cascade-specific camera frustum slice bounds; point and spot shadow views derive perspective cameras from authored light position, direction, range, and cone/face data. All shadow views ignore main-camera layer visibility and filter candidates through `PrimitiveRelevance::shadow_caster()` plus their own frustum result. Occlusion is fixed at zero until VC-M3 wires HZB.

`FrameVisibility` exposes view-key and entity-set helpers. `main_view_visible_entity_set()` is now the source passed into Hybrid GI and Virtual Geometry visibility planning, while `shadow_views()` / `shadow_visible_entity_set()` give shadow consumers a single place to union directional cascade, point-face, and spot shadow results without scanning the raw view vector themselves.

`construct.rs` also derives `visible_instances`, `draw_commands`, and GPU instancing candidates from `VisibilityContext::visible_batches_for_entities(...)` using that same main-view entity set. The public `main_view_visible_batches()` accessor recomputes the same filtered batch view from `batches + FrameVisibility`, so the context no longer stores a duplicated main-view batch list. `collect_batching_result.rs` no longer carries a separate `visible_batches` map, so the frame has one view-authoritative source for main-camera visibility instead of parallel flat collections.

## Mesh Pass Consumption

`FrameSubmissionContext` now carries the computed `VisibilityContext` into both runtime-frame paths. `ViewportRenderFrame` stores the `FrameVisibility` sideband so the renderer can consume visibility without rebuilding it from scene data.

`build_mesh_draws(...)` maps frame primitive indices back to source entities, attaches `PrimitiveRelevance`, main-view visibility, and shadow-view visibility onto each `MeshDraw`, and then forwards those flags into `MeshBatchRef`. The mesh pass processors use that data as the pass-participation gate:

- depth prepass, opaque, alpha-mask, transparent, and motion-vector commands require main-view visibility and the matching relevance bit
- shadow commands require shadow-caster relevance and at least one shadow view containing the primitive
- if no shadow view exists, shadow submission falls back to `shadow_caster` relevance so the existing preview/default-shadow path still has valid caster candidates

`mesh_visibility_states_preserve_shadow_only_casters` guards this handoff at the source mapping layer: a primitive culled from the main camera but visible in a shadow view is recorded with `main_view_visible=false` and `shadow_view_visible=true`, so later shadow processors can still emit the caster.

The older queue profile still determines which material phase and pipeline variant a draw would use. Relevance now decides whether the draw participates in that phase for the current view.

`FrameSubmissionContext::view_visibility(key)` exposes per-view results to submit-time consumers. The Virtual Geometry debug node/cluster cull snapshot now reads the main camera through that accessor, so debug replay follows the same view authority as runtime visibility instead of reaching back to `frame_extract.view.camera` first.

## Visibility Diagnostics

`RenderStats` now exposes frame-level visibility counters derived from `FrameVisibility.views`:

- `last_visibility_view_count`
- `last_visibility_input_count`
- `last_visibility_layer_filtered_count`
- `last_visibility_frustum_culled_count`
- `last_visibility_occlusion_culled_count`
- `last_visibility_visible_count`

`update_base_stats(...)` sums the per-view `ViewCullingStats` rows each submitted frame. The current VC-M1 implementation therefore reports main-view plus directional, point-face, and spot shadow-view CPU culling work; occlusion remains zero until VC-M3 wires HZB/GPU occlusion into the same stats path.

`render_stats_store::product` records those fields under `render.visibility.*`, and the runtime diagnostics fixture asserts the same paths. This gives product diagnostics, devtools snapshots, and future tests a stable place to verify that relevance, layer filtering, frustum culling, and later occlusion are all using the per-view visibility authority.

The VC-M4 static-index path extends the same product surface under `render.visibility.static_index.*`. In addition to frame rebuild/update counts and index size, `RenderStats` and product diagnostics now expose main-view prefilter evidence: `main_view_prefilter_used`, `main_view_static_input_count`, and `main_view_static_candidate_count`. These fields make the 10,000-static-instance grid coarse query observable without inspecting private `VisibilityContext` state.

## HZB Occlusion Foundation

VC-M2 introduces the shared HZB foundation that later GPU occlusion, SSR, and SSAO consumers will use instead of each feature owning private depth preparation. `graphics::visibility::occlusion::HzbBuilder` is the WGPU-free sizing authority. It converts the effective render size into a half-resolution, power-of-two furthest-depth pyramid:

- `1923x1081` becomes `1024x1024`
- the same case produces `11` mip levels
- reduce work is grouped in batches of up to `4` mips per pass, so the example requires `3` reduce passes
- `1x1` remains valid and produces a single mip

The render graph side now has a built-in `BuiltinRenderFeature::Hzb` descriptor. Default 3D pipelines schedule its `hzb-build` pass after shadow work and before clustered lighting on the ambient-occlusion stage. The pass declares executor `visibility.hzb-build`, reads `scene-depth`, writes the storage texture resource `hzb-furthest`, and carries a `RenderGraphComputeWorkload::hzb_furthest(...)` workload so execution audit can compare the planned dispatch extent with the runtime dispatch record.

`compile.rs` materializes `hzb-furthest` as `Rgba16Float` with the HZB builder dimensions and a full mip chain. `RenderGraphComputeWorkloadDispatchContext` has a dedicated `HzbFurthest` extent, so dispatch auditing uses HZB texture dimensions rather than the full viewport or the clustered-light grid. Runtime graph execution records HZB dispatch metadata through `record_hzb_build_to_resource(...)`, validates that both depth and HZB resources are bound, and reports storage writes against `hzb-furthest`.

The WGPU execution path now creates a concrete HZB compute pipeline from `post_process/shaders/hzb_build.wgsl`. `ScenePostProcessResources` owns the HZB bind group layout, uniform parameter buffer, pipeline, and a 1x1 fallback source HZB view used only when writing mip 0. `record_hzb_build_to_resource(...)` walks the `HzbBuildPlan` mip chain, creates a single-mip target view for each HZB level, and dispatches `execute_hzb_build_mip(...)` once per level. Mip 0 reduces 2x2 scene-depth texels into `hzb-furthest` mip 0; mip N reduces 2x2 texels from mip N-1 into mip N. Source coordinates outside the source extent return far depth `1.0`, so power-of-two padding remains conservative for the current 0-near, 1-far depth convention. The shader writes the reduced scalar depth into all RGB channels of an `rgba16float` storage texture and alpha `1.0`.

HZB occlusion culling is separately gated from HZB build. The current `hzb-occlusion-cull` compute pipeline needs `10` storage-buffer bindings visible to the compute stage across scene, occlusion, and GPUScene bind groups. `RenderBackendCaps` and `RenderCapabilitySummary` therefore carry `max_storage_buffers_per_shader_stage`; WGPU offscreen device creation requests the HZB requirement when the adapter supports it, otherwise the renderer still starts but does not construct `HzbOcclusionCuller`. Profile compilation then filters only `visibility.hzb-occlusion-cull`, preserving the HZB build/history resources for SSR/SSAO and diagnostics. Runtime diagnostics expose the raw limit at `render.capability.max_storage_buffers_per_shader_stage`.

Frame history now reserves a matching HZB history texture beside scene color, GI, AO, and SSR history. The texture is sized from the effective render size, tracks the HZB mip count, imports the previous-frame view as `history.previous.hzb-furthest`, and copies the current `hzb-furthest` mip chain into history at frame end when the compiled pipeline writes HZB. `RenderHistoryCopyReport` exposes `hzb_furthest_copied`, and runtime diagnostics record it at `render.history.copy.hzb_furthest_copied`.

`RenderStats` also exposes HZB-specific progress:

- `last_hzb_mip_count`
- `last_hzb_graph_executed_pass_count`
- `last_hzb_occlusion_reported`
- `last_hzb_occlusion_candidate_arg_count`
- `last_hzb_occlusion_candidate_instance_count`
- `last_hzb_occlusion_dispatch_group_count`
- `last_hzb_occlusion_dispatched_phase_count`
- `last_hzb_occlusion_history_available`
- `last_hzb_occlusion_readback_available`
- `last_hzb_occlusion_tested_arg_count`
- `last_hzb_occlusion_tested_instance_count`
- `last_hzb_occlusion_culled_arg_count`
- `last_hzb_occlusion_culled_instance_count`
- `last_hzb_occlusion_indirect_args_readback_available`
- `last_hzb_occlusion_readback_arg_count`
- `last_hzb_occlusion_zero_instance_arg_count`
- `last_hzb_occlusion_remaining_instance_count`

`update_base_stats(...)` derives the mip count from `HzbBuilder`, counts executed `visibility.hzb-*` executors, and copies the latest HZB occlusion report when the pass executes. Product diagnostics record these as `render.hzb.mip_count`, `render.hzb.graph_executed_pass_count`, and `render.hzb.occlusion.*`; the runtime diagnostics fixture asserts the HZB series. Candidate/dispatch fields are execution metadata, while `readback_available` and the tested/culled counters come from the GPU stats buffer. When readback stats are present, `last_visibility_occlusion_culled_count` is overridden with the exact HZB culled instance count.

Current VC-M2 status has moved past the resource-only foundation: the graph resource, history slot, diagnostics, WGPU pipeline, WGSL shader, and per-mip dispatch path are all wired for the shared furthest-depth pyramid. The graph audit still records one aggregate HZB compute dispatch for the feature workload contract, while the actual command encoder dispatches once per mip so each pass can bind the previous mip as the next source view.

SSAO now declares and binds `hzb-furthest` beside scene depth and normals. The SSAO shader samples HZB mip 1 as a coarse depth signal and applies a small conservative occlusion adjustment before the existing history blend. This keeps the old AO output/history contract intact while moving the broad depth hierarchy dependency to the shared resource.

SSR resolve now consumes `hzb-furthest` through binding 23 of the existing post-process bind group. The old variable name remains in WGSL as a compatibility alias, but the runtime passes the full-mip shared HZB view instead of the former private SSR depth pyramid. The fallback for single-mip HZB now samples mip 0 rather than the removed private depth-coarse texture. The private SSR depth-pyramid feature nodes, graph resource names, runtime executors, execute modules, and pipeline bundle entries have been deleted; the reflection-color pyramid remains because it is the rough reflection color cache, not a depth preparation path.

The remaining VC-M2 acceptance work is RenderDoc validation of the HZB mip chain and visual/behavioral SSR/SSAO regression review. One known quality caveat is that HZB is currently a furthest-only `Rgba16Float` chain. The removed SSR private depth pyramid carried a min/max range in `.rg`; the shared path currently gives SSR equal min/max values from the furthest depth value, so reflection hit gating needs frame inspection before VC-M2 is treated as fully accepted.

## VC-M3 GPU Occlusion Runtime Slice

The visibility module itself remains WGPU-free for VC-M3. Its new contract is `HzbOcclusionCullReport`, with `HzbOcclusionPhase::SingleFrameReproject` as the implemented phase and `TwoPhaseRetest` reserved for a later two-stage retest/redraw path. The report carries candidate arg count, candidate instance count, phase-local dispatch group count, dispatched phase count, whether previous HZB history was available, optional `HzbOcclusionCullReadbackStats` copied from the WGPU stats buffer, and optional `HzbOcclusionIndirectArgsReadbackSummary` copied from the phase-local indirect args buffers after the occlusion pass. The dispatch group count is the sum of `ceil(args_count / 64)` for each actually dispatched mesh phase, not `ceil(total_candidate_args / 64)`, because opaque, alpha-mask, and velocity use separate params uploads and separate compute dispatches. `graphics::scene::scene_renderer::hzb::phase_dispatch` owns that phase-local accounting and report summary so `HzbOcclusionCuller` stays focused on WGPU pipeline, bind group, and command encoding work.

The WGPU execution lives under `graphics::scene::scene_renderer::hzb`. `HzbOcclusionCuller` owns the `zircon-hzb-occlusion-cull-pipeline`, a params uniform buffer, a GPU stats storage buffer, a readback buffer, and a bind group layout for:

- previous HZB texture view
- `HzbOcclusionCullParams`
- phase-local indexed-indirect args storage buffer
- immutable compaction metadata storage buffer
- visible-instance remap storage buffer
- per-batch indirect draw-count storage/indirect buffer
- compacted indexed-indirect args storage/indirect buffer
- HZB occlusion stats storage buffer

Per-phase cull params are uploaded with an encoder-scoped COPY_SRC staging buffer before each compute dispatch. This keeps params copy -> dispatch ordering inside the command buffer, so multiple mesh phases cannot all observe the final `args_count` written before submission.

The compute shader includes `zr_gpu_scene.wgsl` and `zr_hzb.wgsl`. It dispatches one thread per indirect args record, walks the record's instance range, projects GPUScene primitive bounds through `SceneUniform.previous_view_proj`, chooses a previous-HZB mip from screen radius, and uses a conservative nearest-depth <= furthest-depth comparison. Each tested args record atomically increments tested arg/instance counters. Visible source instance indices are appended into the phase-local remap buffer, and visible draws are compacted into the batch output range through a per-batch atomic draw-count slot. Fully hidden records do not write compacted args and atomically increment culled arg/instance counters.

The built-in HZB feature now declares two passes:

- `hzb-occlusion-cull` in `DepthPrepass`, executor `visibility.hzb-occlusion-cull`, async-compute declared queue, previous-HZB read, required execution-owned external buffer reads/writes for compaction metadata, indirect args, visible-instance remap, draw-count, and HZB stats, plus `RenderGraphComputeWorkload::indirect_args(...)`.
- `hzb-build` in `AmbientOcclusion`, executor `visibility.hzb-build`, scene-depth read, HZB storage write, and `RenderGraphComputeWorkload::hzb_furthest(...)`.

`RenderGraphComputeWorkloadDispatchContext` has an `IndirectArgs` extent so graph execution audit can compare planned occlusion work against mesh indirect args. The default extent still derives a single 1D group count from total args, but HZB occlusion overrides it with the phase-local dispatch group count from `HzbOcclusionCullReport`. That keeps graph audit aligned with the actual encoder shape when three small phase buffers dispatch as three separate compute workloads. `MeshIndirectDrawExecution` now creates args buffers with `STORAGE` usage and exposes `args_count()` / `total_instances()`, while `RenderPassMeshCommandLists` can report the total occlusion cull candidate arg and instance counts across all mesh phases. `MeshIndirectArgsReadback` and `MeshIndirectArgsSnapshot` provide the post-cull readback parser for real WGPU indirect args buffers, including `zero_instance_arg_count()`, `remaining_instance_count()`, and `compacted_draw_count()` for product diagnostics.

`mesh_pass::IndirectCompactionPlan` is the Rust-side ABI for UE-style clear + atomic compact replay. It is built from the same phase-local `IndexedIndirectArgs` rows that back the WGPU indirect args buffer. Each metadata row preserves the source args record index, source `first_instance/count`, output arg base, draw-count slot, and the prefix-assigned visible-instance remap base. `MeshIndirectCompactionResources` allocates the matching WGPU buffers per `MeshIndirectDrawExecution`: immutable metadata storage, visible-instance remap storage/copy buffer, per-batch storage/copy/indirect draw-count buffer, and compacted indirect args storage/copy/indirect buffer. Before each phase HZB occlusion dispatch, `HzbOcclusionCuller` clears the visible-instance remap allocation, draw-count buffer, and compacted args buffer while leaving metadata immutable. After dispatch, mesh replay uses `multi_draw_indexed_indirect_count(...)` against the compacted args plus draw-count buffer, and binds a phase-local group3 visible-remap scene bind group so shader instance ids resolve back to original GPUScene instances.

Runtime graph execution injects `HzbOcclusionCuller` only into stages that have mesh draw lists. `record_hzb_occlusion_cull_to_indirect_args(...)` uses `history.previous.hzb-furthest` when present and falls back to the post-process white texture view on the first frame, then records compute storage writes against `"mesh.compacted-indirect-args"`, `"mesh.visible-instance-index"`, `"mesh.indirect-draw-count"`, and `"visibility.hzb-occlusion-stats"` for graph audit. Source indirect args and compaction metadata are declared as external reads in the feature descriptor because they are execution-owned inputs for the pass.

The compiled-scene render path now binds those required execution-owned external buffers into `RenderGraphExecutionResources` before materialization validation. `bind_execution_owned_graph_resources(...)` maps live HZB external lifetimes to the first phase-local indirect execution's source/compaction/replay buffers and to `HzbOcclusionCuller::stats_buffer()`. When a graph declares the HZB occlusion pass but the frame has no candidate phases or no culler, the helper creates minimum bindable fallback buffers so the graph materialization report records the HZB external names as bound rather than missing. Because the HZB descriptor now declares these names as `required_buffer`, a missing binding fails validation before the executor can run. This is HZB-specific ownership evidence; the visibility module remains WGPU-free and non-HZB required external ownership still needs the generalized graph split.

The returned `HzbOcclusionCullReport` is preserved through `RenderPassGpuExecutionContext`, `RenderGraphExecutionRecord`, and `SceneRenderer::last_hzb_occlusion_cull_report()`. After graph execution but before encoder submission, `SceneRendererCore` encodes readbacks for the same phase-local replay buffers that submission consumes. After submission, it maps the HZB stats readback buffer, compacted replay args, and draw-count buffers, attaching exact stats plus the indirect args summary to the report when dispatches executed; zero-dispatch reports get exact zero rows. `update_base_stats(...)` copies the report into `RenderStats`, while frames without the pass explicitly reset the HZB occlusion report fields to zero/false. Product diagnostics expose this surface as `render.hzb.occlusion.reported`, `candidate_arg_count`, `candidate_instance_count`, `dispatch_group_count`, `dispatched_phase_count`, `history_available`, `readback_available`, `tested_*`, `culled_*`, `indirect_args_readback_available`, `readback_arg_count`, `compacted_draw_count`, `zero_instance_arg_count`, and `remaining_instance_count`. `graphics::visibility` re-exports `HzbOcclusionCullReport`, `HzbOcclusionCullReadbackStats`, and `HzbOcclusionIndirectArgsReadbackSummary` from the root module so scene-renderer and render-stats consumers do not import through the private occlusion subtree.

Runtime graph compilation now has a backend capability gate for this pass. `RenderCapabilitySummary::hzb_occlusion_culling_supported()` requires storage buffers plus the same GPU-driven submission support used by GS-M4: indirect draw, multi-draw indirect, and indirect first-instance. `compile_options_for_profile(...)` maps that capability into `RenderPipelineCompileOptions::enable_hzb_occlusion_culling`, and `RenderPipelineAsset::compile_with_options(...)` removes only `visibility.hzb-occlusion-cull` when the flag is disabled. The HZB build pass and `hzb-furthest` history resource stay compiled so SSR/SSAO keep their shared HZB input.

This makes the headless WGPU path explicit. The current headless device starts with an empty WGPU feature set, so runtime profile compilation disables HZB occlusion culling and CPU relevance/frustum visibility remains the final visibility result. The advanced post-process submit coverage asserts that the runtime graph does not execute `hzb-occlusion-cull` in that fallback path.

Current limitations are explicit:

- The capability-gated CPU fallback graph path, exact stats aggregation path, indirect args readback summary path, local WGPU wall/front args rewrite path, and product-level wall-scene source assertion are covered.
- The visible-instance remap/compact ABI, atomic compact WGSL, graph declarations, count-buffer replay, group3 remap consumption, compact draw-count diagnostics, and phase-local dispatch audit are wired. Focused HZB culler, phase-dispatch, indirect-compaction, indirect-draw-execution, descriptor external, and multi-draw replay tests now have clean local evidence; the HZB product wall-scene clean rerun remains open.
- HZB executor-owned external buffers are now materialized into the graph execution table as required buffers. The fallback-buffer source test, descriptor required-binding assertion, and required-external validation test are authored but still need focused runs.
- `RenderStats.last_visibility_occlusion_culled_count` is now backed by HZB GPU stats readback when the pass executes. `render_product_advanced.rs::render_product_hzb_occlusion_wall_scene` consumes `zero_instance_arg_count`/`remaining_instance_count` and the HZB readback surface for a wall + 64 hidden static-instance scene. It now also renders the same two-frame scene through a capability-gated CPU fallback baseline and asserts captured RGBA equality against the HZB occlusion path. The remaining product acceptance gap is a clean lib-test run.
- RenderDoc validation of the occlusion cull dispatch and HZB history input is still pending.

## VC-M4 Static Index Core

`graphics::visibility::static_index::VisibilityStaticIndex` is the first VC-M4 static-scene spatial-index slice. It remains WGPU-free and consumes the existing `VisibilityBvhInstance` rows plus `VisibilityBvhUpdatePlan` diffs rather than introducing a new render-resource ABI.

The current structure is a single-level uniform grid. Each static entity stores its bounds and the cell coordinates touched by its bounding sphere. The grid maps each cell back to a sorted entity set, so `query_bounds(...)` can collect deterministic entity ids for a query sphere. `rebuild(...)` replaces the whole table, while `apply_update_plan(...)` uses the existing update strategy:

- `FullRebuild` clears and rebuilds from the provided current instance list.
- `Incremental` removes deleted entities first, then inserts or replaces inserted/updated entities found in the current instance list.
- if an incremental plan references a missing inserted/updated entity, the index removes that entity to keep stale rows from surviving.

`VisibilityStaticIndexReport` tracks cumulative rebuild/update counters, per-frame rebuild/update counters, inserted/updated/removed counts, indexed entity count, occupied cell count, and main-view static prefilter evidence. The prefilter evidence records whether the 10,000 static-instance threshold path was used, how many static instances entered the frame, and how many static candidates survived the grid coarse query. The local tests compare incremental update query results against a fresh full rebuild and verify that a full-rebuild strategy replaces old rows.

The index now has a per-viewport persistent owner. `ViewportRecord` stores the latest `VisibilityStaticIndex`; `resolve_viewport_record_state(...)` clones it beside the existing previous `VisibilityHistorySnapshot`; `VisibilityContext::from_extract_with_history_and_static_index(...)` applies the current frame's static-only `VisibilityBvhInstance` rows against the BVH update plan; and `record_history(...)` writes the resulting index back after a successful submission history update. If a previous visibility history exists but the previous static index is missing, the context forces one full static-index rebuild so unchanged static rows are not lost.

Main-view culling now uses the persistent index for static/dynamic split. Static sets below 10,000 instances keep the old linear path. Static sets at or above 10,000 instances first query the static grid with a conservative world-space camera bounds sphere, then pass only those static candidates plus all dynamic main-view candidates into the existing bounds-level frustum refinement. This preserves the old `mesh_frustum_visibility(...)` narrow-phase behavior while avoiding full static-scene linear scans for large static worlds. `graphics/tests/render_framework_visibility_submit.rs::render_framework_reuses_static_index_and_reports_main_view_prefilter` now provides the render-framework static-scene source assertion: two consecutive 10,001-static-mesh submissions must report second-frame `frame_full_rebuild_count == 0` and a main-view static candidate count below static input count. Its first run exposed an older scene-renderer debug assert that still required per-phase command counts to equal the source draw census; with visibility pruning, command counts are allowed to be lower, so the assert now enforces only `command_count <= source_draw_count`. The scoped rerun and direct `visibility` sweep both passed on 2026-06-15; remaining VC-M4 acceptance is full render-product regression and RenderDoc visual review.

2026-06-15 update: the scoped clean rerun is no longer blocked. `cargo test -p zircon_runtime render_framework_reuses_static_index_and_reports_main_view_prefilter --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never -- --test-threads=1 --nocapture` passed 1 filtered test with the existing warning set. A follow-up direct run through the built core-min lib-test binary first exposed the graphics root source-shape guard for `ViewportRenderFrame`; after keeping `pub(crate) use types::ViewportRenderFrame;` as an independent re-export and keeping `ViewportRenderOutputTarget` crate-private separately, the exact guard test passed and the same binary's `visibility --test-threads=1 --nocapture` filter passed 59/59. Remaining VC-M4 acceptance is now full render-product regression and RenderDoc visual review.

## Integration Boundaries

`PrimitiveRelevance` lives under `core::framework::render` because it is a renderer-neutral product contract. It does not know about WGPU buffers, mesh pipelines, command replay, or render graph resources.

`parallel_frustum.rs` lives under `graphics::visibility::culling` because it is an implementation detail of the CPU culling pipeline. It consumes `MeshFrustumCandidate` rows built from `VisibilityBounds`, routes large-scene parallelism through the runtime-owned compute `TaskPool` and `parallel_for(...)`, and uses `is_bounds_visible(...)` as the shared frustum kernel.

This bounds-level kernel is the bridge toward the linear array model described in plan 04: visibility can now evaluate extracted bounds without holding mesh snapshot references, while `VisibilityBvhInstance` and history entries continue to receive the same precomputed bounds for compatibility.

`VisibilityStaticIndex` stays under `graphics::visibility::static_index` because it is a CPU-side acceleration structure for static primitive bounds. It uses `VisibilityBvhUpdatePlan` as its update input, but it is intentionally not a render graph resource, GPUScene resource, or WGPU buffer owner. The render-framework ownership layer stores only the CPU index in `ViewportRecord`; WGPU resource lifetime remains untouched.

`VisibilityContext` no longer exposes the pre-existing single-view storage fields. The per-view `FrameVisibility` / `ViewVisibilityContext` shape is present for the main camera, directional cascades, point faces, spot shadows, and scene-authored custom target cameras. Main-camera compatibility queries are explicit derived methods instead of stored fields. Directional cascade views now use Plan 05 CSM frustum-slice bounds instead of sharing one frame-bounds light camera. For scene-backed extracts, visibility consumes non-primary-target `RenderViewExtract.cameras` descriptors to build `VisibilityViewKey::CustomTarget { camera }` rows, while `RenderCameraOrderReport` remains ordering/diagnostic evidence. This is a CPU visibility descriptor bridge only: WGPU still submits one effective camera until plan 09 lands the multi-camera render loop, target-output ownership, and per-camera post/history/light rules.

## Validation State

Formatting passed for all touched Rust files.

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` passed after the relevance/frustum integration, bounds-kernel follow-up, main-view `FrameVisibility` integration, shadow-view integration, mesh-pass relevance consumption, main-view planning accessor migration, and visibility RenderStats/diagnostics integration, with the repository's existing warning set.

The scoped core-min check passed again after the shadow-view atlas expansion: `VisibilityViewKey` now covers `ShadowCascade`, `ShadowPointFace`, and `ShadowSpot`; `FrameVisibility::from_frame_views(...)` builds all Plan 05 shadow atlas view keys currently required by directional/point/spot slots; and the source test `visibility_context_builds_shadow_views_for_atlas_light_slots` asserts the 4 + 6 + 1 view-key surface plus distinct directional cascade camera sizing. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` passed with the repository warning set. The directional cascade slice follow-up compiled the shared lib-test target with `--no-run`, then ran `cargo test ... visibility_context_builds_shadow_views_for_atlas_light_slots -- --nocapture` successfully as one filtered test. A prior `--exact` attempt matched zero tests because the full module path was required, so it is not counted as coverage.

The scoped core-min check passed again after the LS-M3 caster/receiver source guard: `mesh_visibility_states_preserve_shadow_only_casters` now proves that `build_mesh_draws(...)` preserves shadow-only casters when main-view culling excludes them. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-caster-receiver-coremin --message-format short --color never` passed with the repository warning set. The focused `cargo test ... mesh_visibility_states_preserve_shadow_only_casters -- --exact --nocapture` attempt timed out after 904 seconds without a filtered result; matching target-dir cargo/rustc processes were stopped.

The 2026-06-18 VC-M3 focused rerun fixed the HZB culler source-contract test so it scans only the implementation section before comparing clear-before-dispatch ordering. `cargo test -p zircon_runtime --lib hzb_occlusion_culler --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-culler-contract-0618 --message-format short --color never -- --test-threads=1 --nocapture` passed 6/6, including the real offscreen WGPU wall/front cull test and clear-before-dispatch source contract. The same built lib-test binary also passed `hzb_occlusion_dispatch` 4/4, `indirect_compaction` 8/8, `mesh_indirect_draw_execution` 3/3, `multi_draw_indexed_indirect` 1/1, and `hzb_occlusion_cull_declares_execution_owned_external_buffers` 1/1. `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-culler-contract-0618` passed with the existing warning set; an earlier parallel check invocation on the same target-dir timed out in the tool window but finished naturally before the hot rerun.

The same scoped check passed after the VC-M2 HZB builder, graph descriptor/resource, runtime dispatch record, HZB history texture/copy, and HZB diagnostics integration. The command returned the repository's existing warning set.

The VC-M2 touched Rust files passed `rustfmt --edition 2021 --check`. A trailing-whitespace scan over the HZB code/docs/session files returned clean, and `git diff --check` over the same scoped file list exited 0 with only Git's LF-to-CRLF notices.

The legacy main-view field cutover removed `VisibilityContext.visible_entities`, `culled_entities`, and `visible_batches` after replacing their callers with derived `main_view_*` methods. The scoped formatting gate passed for `visibility_context.rs`, `construct.rs`, and `graphics/tests/visibility.rs`; `git diff --check` over the visibility cutover files and docs exited 0 with only Git's LF-to-CRLF notices; and `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-visibility-legacy-fields-0617` completed with the repository's existing warning set. A cross-crate scan of `zircon_app`, `zircon_editor`, `zircon_runtime_interface`, and `zircon_runtime/tests` found no remaining direct reads of the removed fields.

The 2026-06-18 custom-target visibility descriptor bridge keeps `FrameVisibility` as the primitive-index authority while adding per-primitive `render_layer_masks`. Scene extract unions the selected scene camera descriptor layers with Texture/Headless descriptor layers only for mesh and sprite candidates, then custom-target view construction applies each descriptor's own layer mask with `PrimitiveRelevance::view_visible_for_layers(...)`. The main-view relevance pass now reads `RenderViewExtract::selected_camera_layers()` as well, aligning the default view with the same descriptor-owned layer source. The earlier scoped core-min check passed with `D:\cargo-targets\zircon-runtime-custom-target-visibility-0618`, the lib-test binary compiled with `--no-run`, and direct exact binary tests passed for the original camera ordering payload bridge. The descriptor-consumer cutover later passed `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-descriptor-visibility-0618 --message-format short --color never`, lib-test `--no-run`, and direct exact binary tests for descriptor-driven custom-target visibility, scene extract custom-target candidate preservation, and scene camera descriptor/report extraction. RenderDoc validation was not run because the MCP bridge reported no running RenderDoc instances.

The scoped check passed again after the concrete WGPU HZB build path was added: shader module creation, HZB bind group layout, parameter buffer, fallback source texture view, compute pipeline, per-mip texture views, and `execute_hzb_build_mip(...)` dispatch wiring all type-check under `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`. That run reported 90 existing warnings.

The scoped check passed again after migrating SSR/SSAO to the shared HZB consumer path and deleting the private SSR depth-pyramid production path. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-verify --message-format short --color never` completed in an isolated validation target with the repository warning set; the latest run reported 65 existing warnings.

The scoped check passed again after the VC-M3 WGPU occlusion runtime slice: indirect-args workload auditing, storage-capable mesh indirect args buffers, `HzbOcclusionCuller`, `visibility.hzb-occlusion-cull` executor registration, previous-HZB fallback, and default pipeline pass-order expectations all type-check under `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3 --message-format short --color never`. That run reported the repository warning set. During this validation, an unrelated dirty `zircon_runtime/src/scene/ecs/events.rs` borrow conflict blocked the package check; the fix only hoisted two `len()` reads before mutable vector borrows in `shrink_buffers_to(...)`.

After the HZB shader-slice documentation update, `rustfmt --edition 2021 --check` passed for the HZB touched Rust file list, the scoped HZB code/docs/session trailing-whitespace scan returned clean, and `git diff --check -- <HZB scoped tracked files>` exited 0 with only Git's LF-to-CRLF notices.

The scoped check passed again after the VC-M3 capability gate and headless CPU fallback graph path: `rustfmt --edition 2021 --check` passed for the touched HZB gate Rust file list, scoped `git diff --check` and trailing-whitespace scans returned clean with only Git LF-to-CRLF notices, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-gate --message-format short --color never` completed with the repository warning set. That run reported 66 existing warnings.

The scoped check passed again after the VC-M3 HZB occlusion report surface: `rustfmt --edition 2021 --check` passed for the touched Rust file list, `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-report --message-format short --color never` completed with the repository warning set, and the run reported 66 existing warnings. `cargo test -q -p zircon_runtime --lib update_hzb_occlusion_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-report-tests -- --test-threads=1 --nocapture` passed 2 filtered tests.

The scoped check passed again after the VC-M3 exact stats readback path: `rustfmt --edition 2021 --check` passed for the touched Rust file list, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-readback --message-format short --color never` completed with the repository warning set, reporting 66 existing warnings. A filtered `cargo test -q -p zircon_runtime --lib update_hzb_occlusion_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-readback -- --test-threads=1 --nocapture` attempt printed `3 passed; 0 failed; 3790 filtered out` but the command wrapper timed out during the shared lib-test process tail. A retry was blocked before filtered tests ran by unrelated plugin-session test target errors in `host_api_adapter.rs` and `native_plugin_live_host.rs`.

The scoped format and whitespace checks passed again after the VC-M3 indirect args readback summary path: `rustfmt --edition 2021 --check` passed for the touched Rust file list, and scoped `git diff --check` exited 0 with only Git LF-to-CRLF notices. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-args-readback-coremin --message-format short --color never` reached `zircon_runtime` but failed on an unrelated task-module visibility error: `zircon_runtime/src/core/runtime/tasks/mod.rs` re-exports private `JobSchedulerDiagnosticsState`. No render visibility error was returned before that blocker.

The scoped checks passed again after the VC-M3 local WGPU wall/front args rewrite test and params-upload ordering fix: `rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs` passed, scoped `git diff --check` and trailing-whitespace scans passed with only Git LF-to-CRLF notices, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-wgpu-local-coremin --message-format short --color never` completed with the repository warning set, reporting 74 existing warnings.

Focused lib-test execution still has no clean result for this latest slice. `cargo test -p zircon_runtime --lib hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-wgpu-local-test --message-format short --color never -- --test-threads=1 --nocapture` timed out after 10 minutes without returning a filtered test result. `cargo test -p zircon_runtime --lib hzb_occlusion_uploads_phase_params_in_encoder_order --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-wgpu-local-coremin --message-format short --color never -- --test-threads=1 --nocapture` timed out after 15 minutes while still compiling the shared `zircon_runtime` lib-test target. The leftover cargo/rustc processes for those two target dirs were stopped.

The 2026-06-13 VC-M3 product wall-scene source assertion follow-up added `render_product_hzb_occlusion_wall_scene` in `graphics/tests/render_product_advanced.rs`. `rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_advanced.rs` passed, and the scoped `git diff --check` passed with only Git LF-to-CRLF notices. `cargo test -p zircon_runtime render_product_hzb_occlusion_wall_scene --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-product-static-diagnostics` reached the shared lib-test target but was blocked by unrelated plugin test code at `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs:265:30`: `interface_exports_owned_by` is private. No render visibility error was returned before that blocker. The follow-up source assertion now also compares captured RGBA output against the capability-gated CPU fallback baseline; `rustfmt --edition 2021 --check`, scoped `git diff --check`, and trailing-whitespace scan passed for `render_product_advanced.rs`. Cargo was not restarted for that extension because active editor UI and plugin reload Cargo lanes were already compiling `zircon_runtime`.

The 2026-06-13 VC-M3 indirect compaction ABI follow-up passed `rustfmt --edition 2021 --check` for `mesh_pass/indirect_compaction.rs`, `mesh_pass/indirect_draw_execution.rs`, and `mesh_pass/mod.rs`. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-coremin --message-format short --color never` passed with the repository warning set, reporting 69 existing warnings. A focused `cargo test -p zircon_runtime --lib indirect_compaction --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-coremin --message-format short --color never -- --test-threads=1 --nocapture` attempt timed out after 424 seconds while compiling the shared lib-test target. The leftover cargo/rustc processes for that target dir were stopped, and no filtered test result was returned.

The 2026-06-13 VC-M3 indirect compaction resource follow-up added per-execution WGPU metadata, visible-remap, and draw-count buffers for the existing compact ABI. `rustfmt --edition 2021 --check` passed for the touched mesh-pass Rust files; scoped `git diff --check` and trailing-whitespace scans passed with only LF-to-CRLF notices. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never` passed with 68 existing warnings. `cargo test -p zircon_runtime --lib indirect_compaction_resources --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never -- --test-threads=1 --nocapture` failed before filtered render tests ran because unrelated UI test code in `zircon_runtime/src/ui/component/state_reducer/keyboard.rs:311` requires `String: Borrow<&str>` during shared lib-test compilation. No render visibility error was returned before that blocker.

The 2026-06-13 VC-M3 compaction clear/resource-declaration follow-up added HZB feature external resource declarations for execution-owned metadata, indirect args, visible-instance remap, draw-count, and stats resources; runtime dispatch records now include the three storage outputs touched by HZB culling; and each dispatched phase clears visible-remap and draw-count buffers before the current V1 occlusion shader runs. Source tests cover the descriptor external resources, dispatch-record write list, and clear-before-dispatch ordering. The touched Rust files passed `rustfmt --edition 2021 --check`; scoped `git diff --check` and trailing-whitespace scans passed with only LF-to-CRLF notices; `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never` passed with 68 existing warnings. A focused `cargo test -p zircon_runtime --lib hzb_occlusion_culler_clears_compaction_outputs_before_culling_dispatch --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-indirect-compaction-resources-coremin --message-format short --color never -- --test-threads=1 --nocapture` attempt timed out after 304 seconds while compiling the shared lib-test target; leftover cargo/rustc processes for that target dir were stopped, and no filtered result was returned.

The 2026-06-13 VC-M3 compact replay follow-up changed HZB occlusion from in-place source-args mutation to source/read plus compacted-output writes. The HZB descriptor now treats source indirect args and compaction metadata as read resources, while visible-instance remap, per-batch draw count, compacted indirect args, and stats are write resources. `HzbOcclusionCuller` clears all three compaction outputs before dispatch, writes compacted args with per-batch atomic draw-count slots, then marks the execution ready for replay. Mesh replay uses `multi_draw_indexed_indirect_count` plus a group3 visible-remap bind group for opaque, alpha-mask, and velocity phases only; shadow, depth-prepass, transparent, and command-local palette draws stay direct for correctness. `rustfmt --edition 2021 --check` passed for the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` passed with 68 existing warnings. The focused `indirect_compaction` lib-test filter is still blocked before filtered render tests run by unrelated duplicate `register` definitions in plugin runtime test compilation.

The 2026-06-13 compact draw-count diagnostics follow-up made the HZB indirect args readback summary observe the actual compact replay count buffer. `MeshIndirectArgsReadback` copies the replay args buffer and, when compaction is ready, the draw-count buffer; `HzbOcclusionIndirectArgsReadbackSummary` carries `compacted_draw_count`; `RenderStats` exposes `last_hzb_occlusion_compacted_draw_count`; and product diagnostics records `render.hzb.occlusion.compacted_draw_count`. `render_product_hzb_occlusion_wall_scene` now asserts that compact draw count is nonzero and does not exceed the readback arg capacity. `rustfmt --edition 2021 --check` passed for the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` passed with 68 existing warnings. Focused lib-test execution for `hzb_occlusion_indirect_args_summary_saturates_totals` timed out twice while compiling the shared lib-test target; residual cargo/rustc processes were stopped. RenderDoc validation was not run because no RenderDoc instance was available.

The 2026-06-17 phase-dispatch helper split moved HZB occlusion phase accounting from `hzb_occlusion_culler.rs` into `phase_dispatch.rs`. `HzbOcclusionPhaseDispatch` stores the execution reference, phase-local args count, and derived workgroup count; `HzbOcclusionPhaseDispatchSummary` saturating-accumulates dispatched phases and workgroups for `HzbOcclusionCullReport`. The culler now creates one phase dispatch per non-empty mesh indirect execution, encodes clears and compute work from that typed record, marks compaction ready after dispatch, and reports the helper's summary. This reduces the culler file from 904 to 864 lines and keeps future two-phase retest/report work out of the WGPU pipeline owner. Validation for this slice is lightweight only: rustfmt, scoped diff checks, and conflict-marker scans passed. The first scoped `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-phase-dispatch-0617` timed out after 304 seconds without a compiler diagnostic and its cargo/rustc processes were stopped; the same command passed on a 600-second rerun in 153.4 seconds with only the repository warning set.

The 2026-06-17 required External binding contract follow-up marks the HZB occlusion execution-owned resources as `required_buffer` in the feature descriptor and verifies the metadata survives pipeline compile into the graph lifetime. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-binding-contract-0617 --message-format short --color never` passed with the existing warning set. Focused HZB descriptor, pipeline compile, fallback-buffer, and required-external validation tests remain deferred for the implementation-first phase.

The 2026-06-18 VC-M3 product clean rerun closed the wall-scene product gap. A first direct run of the existing hot lib-test binary reproduced the product failure as a graph declaration bug: `post.uber` referenced `light-list` through its post-process bind group while the compiled pass did not declare that resource. The shared support fix makes `PostProcessStackDescriptor` treat `LIGHT_LIST` as a basic frame resource and makes the `post.uber` feature descriptor explicitly read the buffer. Default forward+ now reads the clustered-lighting graph buffer; clustered-lighting-disabled product profiles keep `light-list` as the renderer-owned External cluster buffer. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-postprocess-light-list-check-0618 --message-format short --color never` passed with the existing warning set. `cargo test -p zircon_runtime --lib render_product_hzb_occlusion_wall_scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-hzb-product-light-list-0618 --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 after the fix, and the same warmed binary passed a hot `render_product_hzb_occlusion_wall_scene` rerun 1/1. The same lane passed `light_list` 4/4, `hzb_occlusion_culler` 6/6, `hzb_occlusion_dispatch` 4/4, `indirect_compaction` 8/8, `mesh_indirect_draw_execution` 3/3, and `multi_draw_indexed_indirect` 1/1. RenderDoc validation is still pending.

The VC-M4 static-index persistent-state and main-view prefilter slice passed scoped formatting and focused tests. `rustfmt --edition 2021` completed for the touched visibility/render-framework/stat/diagnostic Rust files. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-static-index-prefilter --message-format short --color never` passed with the repository warning set, reporting 68 warnings. `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-static-index-state-test --message-format short --color never static_index -- --nocapture` passed 6 filtered tests, covering the static index core, persistent-index reuse, missing-index rebuild, 10,000 static-instance prefilter activation, and stats mapping. `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-static-index-state-test --message-format short --color never render_product_diagnostics_record_visibility_stats -- --nocapture` passed 1 filtered product-diagnostics test. `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-static-index-state-test --message-format short --color never runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins -- --nocapture` now runs but fails on an unrelated light diagnostic expectation: `render.light.point.ready_count` expected `0.0` while the fake render framework reports `4.0`.

The visibility root re-export repair passed as part of the 2026-06-13 editor UI keyboard validation. The first scoped `cargo check` was blocked because `HzbOcclusionCullReadbackStats` was defined under `graphics::visibility::occlusion` but not exported from `graphics::visibility`; after adding the root re-export, `rustfmt --edition 2021 --check zircon_runtime\src\graphics\visibility\mod.rs ...` passed and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-keyboard-routes-0613-coremin --message-format short --color never` passed with the repository warning set. The follow-up grouped-selection catalog test exposed the same boundary for `HzbOcclusionIndirectArgsReadbackSummary`; exporting it from `graphics::visibility` keeps scene-renderer and render-stats consumers on the documented root surface instead of the private occlusion subtree.

The 2026-06-13 main-view prefilter diagnostics follow-up passed scoped formatting and whitespace checks. `rustfmt --edition 2021 --check` passed for the touched render stats, diagnostics, runtime-diagnostics, render-framework visibility submit, and scene-renderer render files; scoped `git diff --check` and the trailing-whitespace scan passed with only Git LF-to-CRLF notices. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc4-product-static-diagnostics --message-format short --color never` passed once after the new `RenderStats`/diagnostic fields, reporting 68 existing warnings. `update_visibility_static_index_stats_records_latest_report` passed 1 filtered test after the shared lib-test target finished compiling; `render_product_diagnostics_record_visibility_stats` passed 1 filtered test through that compiled lib-test binary. `render_framework_reuses_static_index_and_reports_main_view_prefilter` initially failed on the stale mesh queue debug assert described above; after changing that assertion to accept visibility-pruned command counts, the 2026-06-15 scoped rerun passed 1 filtered test with the existing warning set.

The 2026-06-15 scoped clean rerun for `render_framework_reuses_static_index_and_reports_main_view_prefilter` passed 1 filtered test in `D:\cargo-targets\zircon-runtime-temporal-s4d-0614`, with 44 existing warnings. The direct lib-test binary sweep then passed `visibility` 59/59 after the `graphics/mod.rs` crate-private re-export split described above. This closes the stale mesh-queue debug-assert and broad visibility verification gaps for VC-M4; render-product and RenderDoc gates are still pending.

`cargo test -p zircon_runtime --lib hzb --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` did not run the filtered HZB tests because the shared lib-test target currently fails to compile first in unrelated plugin extension bridge test code: `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` imports missing `crate::plugin::{BridgeInterfaceSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionReport}`.

`cargo test -p zircon_runtime --lib hzb_occlusion --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain-vc3-tests -- --nocapture` did not return a filtered VC-M3 test result; it timed out after 904 seconds while compiling the shared lib-test target, and the leftover cargo/rustc processes from that attempt were stopped.

Focused lib-test coverage has not returned a clean result yet. One attempt failed before running the filtered tests because unrelated lib-test sources referenced a missing `RuntimePluginDescriptor::with_target_mode`; the latest attempt timed out after 304 seconds while compiling the shared `zircon_runtime` lib-test target. No render visibility test failure was returned.

Those files and long-running test target compilation are outside the render visibility slice and were not changed here. The new source tests are present and should be rerun when the shared lib-test target is buildable within the local time budget.
