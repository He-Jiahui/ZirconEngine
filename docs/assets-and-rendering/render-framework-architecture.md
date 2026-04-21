---
related_code:
  - zircon_rhi/src/lib.rs
  - zircon_rhi/src/capabilities.rs
  - zircon_rhi/src/descriptors.rs
  - zircon_rhi/src/device.rs
  - zircon_rhi_wgpu/src/lib.rs
  - zircon_rhi_wgpu/src/capabilities.rs
  - zircon_rhi_wgpu/src/device.rs
  - zircon_render_graph/src/lib.rs
  - zircon_render_graph/src/builder.rs
  - zircon_render_graph/src/graph.rs
  - zircon_render_graph/src/types.rs
  - zircon_framework/src/lib.rs
  - zircon_manager/src/resolver.rs
  - zircon_framework/src/render/framework.rs
  - zircon_framework/src/render/backend_types.rs
  - zircon_framework/src/render/camera.rs
  - zircon_framework/src/render/frame_extract.rs
  - zircon_framework/src/render/overlay.rs
  - zircon_framework/src/render/scene_extract.rs
  - zircon_resource/src/id.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/render_extract/mod.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world/render.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_scene/tests/viewport_packet.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/host/module_host/mod.rs
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/history/mod.rs
  - zircon_graphics/src/runtime/offline_bake/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/normalized_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/build_prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/snapshot.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/mod.rs
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/render_framework/mod.rs
  - zircon_graphics/src/runtime/render_framework/capability_summary/mod.rs
  - zircon_graphics/src/runtime/render_framework/capture_frame/mod.rs
  - zircon_graphics/src/runtime/render_framework/destroy_viewport/mod.rs
  - zircon_graphics/src/runtime/render_framework/compile_options_for_profile/mod.rs
  - zircon_graphics/src/runtime/render_framework/compiled_feature_names/mod.rs
  - zircon_graphics/src/runtime/render_framework/create_viewport/mod.rs
  - zircon_graphics/src/runtime/render_framework/query_stats/mod.rs
  - zircon_graphics/src/runtime/render_framework/queue_capability/mod.rs
  - zircon_graphics/src/runtime/render_framework/reload_pipeline/mod.rs
  - zircon_graphics/src/runtime/render_framework/render_framework_impl/mod.rs
  - zircon_graphics/src/runtime/render_framework/set_pipeline_asset/mod.rs
  - zircon_graphics/src/runtime/render_framework/set_quality_profile/mod.rs
  - zircon_graphics/src/runtime/render_framework/viewport_record/mod.rs
  - zircon_graphics/src/runtime/render_framework/wgpu_render_framework_new/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/build_frame_submission_context/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/virtual_geometry/build_virtual_geometry_prepare.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/feature/builtin_render_feature_descriptor/mod.rs
  - zircon_graphics/src/feature/builtin_render_feature_descriptor/dispatch/mod.rs
  - zircon_graphics/src/material/mod.rs
  - zircon_graphics/src/shader/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/mod.rs
  - zircon_graphics/src/visibility/culling/mod.rs
  - zircon_graphics/src/visibility/culling/is_mesh_visible.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_cluster.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_page_upload_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/mod.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/mod.rs
  - zircon_graphics/src/extract/mod.rs
  - zircon_graphics/src/extract/history.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/create_depth_texture/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_history/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_target/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/target_extent/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input/gpu_pending_request_input.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/scene/scene_renderer/deferred/mod.rs
  - zircon_graphics/src/scene/scene_renderer/history/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_draw_refs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/virtual_geometry_indirect_args_gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/particle.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
- zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/constants/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/clear_render_target/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/fallback_texture/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/params/bloom_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_trace_region_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/reflection_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_trace_regions/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_reflection_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/construct/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/buffer_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/fallback_texture_views/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/pipeline_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/create_buffer_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/bloom.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_editor/src/editing/viewport/controller/mod.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/editor_event/runtime.rs
- zircon_editor/src/host/slint_host/viewport/mod.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/prepare_frame/build_scene_prepare_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/hybrid_gi/build_hybrid_gi_scene_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_buffers.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_bind_group.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/queue_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_rhi/src/lib.rs
  - zircon_rhi/src/capabilities.rs
  - zircon_rhi/src/descriptors.rs
  - zircon_rhi/src/device.rs
  - zircon_rhi_wgpu/src/lib.rs
  - zircon_rhi_wgpu/src/capabilities.rs
  - zircon_rhi_wgpu/src/device.rs
  - zircon_render_graph/src/lib.rs
  - zircon_render_graph/src/builder.rs
  - zircon_render_graph/src/graph.rs
  - zircon_render_graph/src/types.rs
  - zircon_framework/src/lib.rs
  - zircon_manager/src/resolver.rs
  - zircon_framework/src/render/framework.rs
  - zircon_framework/src/render/backend_types.rs
  - zircon_framework/src/render/camera.rs
  - zircon_framework/src/render/frame_extract.rs
  - zircon_framework/src/render/overlay.rs
  - zircon_framework/src/render/scene_extract.rs
  - zircon_resource/src/id.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/render_extract/mod.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world/render.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_scene/tests/viewport_packet.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/host/module_host/mod.rs
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/history/mod.rs
  - zircon_graphics/src/runtime/offline_bake/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/normalized_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/build_prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/mod.rs
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/render_framework/mod.rs
  - zircon_graphics/src/runtime/render_framework/capability_summary/mod.rs
  - zircon_graphics/src/runtime/render_framework/capture_frame/mod.rs
  - zircon_graphics/src/runtime/render_framework/destroy_viewport/mod.rs
  - zircon_graphics/src/runtime/render_framework/compile_options_for_profile/mod.rs
  - zircon_graphics/src/runtime/render_framework/compiled_feature_names/mod.rs
  - zircon_graphics/src/runtime/render_framework/create_viewport/mod.rs
  - zircon_graphics/src/runtime/render_framework/query_stats/mod.rs
  - zircon_graphics/src/runtime/render_framework/queue_capability/mod.rs
  - zircon_graphics/src/runtime/render_framework/reload_pipeline/mod.rs
  - zircon_graphics/src/runtime/render_framework/render_framework_impl/mod.rs
  - zircon_graphics/src/runtime/render_framework/set_pipeline_asset/mod.rs
  - zircon_graphics/src/runtime/render_framework/set_quality_profile/mod.rs
  - zircon_graphics/src/runtime/render_framework/viewport_record/mod.rs
  - zircon_graphics/src/runtime/render_framework/wgpu_render_framework_new/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/build_frame_submission_context/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/virtual_geometry/build_virtual_geometry_prepare.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/mod.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/feature/builtin_render_feature_descriptor/mod.rs
  - zircon_graphics/src/feature/builtin_render_feature_descriptor/dispatch/mod.rs
  - zircon_graphics/src/material/mod.rs
  - zircon_graphics/src/shader/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/mod.rs
  - zircon_graphics/src/visibility/culling/mod.rs
  - zircon_graphics/src/visibility/culling/is_mesh_visible.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_cluster.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_page_upload_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/mod.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/mod.rs
  - zircon_graphics/src/extract/mod.rs
  - zircon_graphics/src/extract/history.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/create_depth_texture/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_history/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_target/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/target_extent/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input/gpu_pending_request_input.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/scene/scene_renderer/deferred/mod.rs
  - zircon_graphics/src/scene/scene_renderer/history/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
- zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
- zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
- zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
- zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
- zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/particle.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
- zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/constants/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/clear_render_target/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/fallback_texture/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/params/bloom_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_trace_region_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/reflection_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_trace_regions/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_reflection_probes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/construct/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/buffer_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/fallback_texture_views/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/pipeline_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/create_buffer_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/bloom.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_editor/src/editing/viewport/controller/mod.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/editor_event/runtime.rs
- zircon_editor/src/host/slint_host/viewport/mod.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/prepare_frame/build_scene_prepare_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/hybrid_gi/build_hybrid_gi_scene_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_buffers.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_bind_group.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/queue_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
plan_sources:
  - user: 2026-04-16 implement Zircon SRP/RHI Rendering Architecture Roadmap
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history-remaining.md
  - docs/superpowers/plans/2026-04-16-m4-runtime-shader-resource-paths.md
  - docs/superpowers/plans/2026-04-16-m4-deferred-runtime-execution.md
  - docs/superpowers/plans/2026-04-16-m4-remaining-behavior-layers.md
  - docs/superpowers/plans/2026-04-16-m5-flagship-capability-slots.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-preprocess.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-prepare-consumption.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-feedback-streaming.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-cluster-refine.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-temporal-radiance-cache-update.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-runtime-bootstrap-removal.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-normalized-multi-region-gather.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-screen-probe-trace-support-resolve.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-size-aware-streaming-uploader.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-shared-indirect-args-buffer.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-streaming-aware-refine-frontier.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-table-confirmed-completion-cascade.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-completion-budget-cascade-closure.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-first-unique-gpu-completion-truth.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-first-unique-page-table-truth.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-page-table-slot-reassignment-normalization.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-first-unique-request-order-closure.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-authoritative-indirect-submission-order.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-submission-index-gpu-args-authority.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-scheduled-trace-region-dedup-closure.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-runtime-resolve-gpu-prepare-rt-lighting-continuation.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-requested-lineage-irradiance-runtime-source.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-requested-lineage-rt-runtime-source.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-feedback-completion-budget-dedup.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-first-unique-gpu-cache-truth.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-explicit-draw-ref-authority-and-cluster-raster-submission.md
  - docs/superpowers/plans/2026-04-20-m5-virtual-geometry-gpu-submission-subset-source.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-scene-driven-parent-chain-runtime-gather.md
tests:
  - zircon_rhi/src/tests/capabilities.rs
  - zircon_rhi/src/tests/descriptors.rs
  - zircon_rhi_wgpu/src/tests.rs
  - zircon_render_graph/src/tests/ordering.rs
  - zircon_render_graph/src/tests/cycles.rs
  - zircon_framework/src/tests.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_scene/tests/viewport_packet.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state.rs
- zircon_editor/src/host/slint_host/viewport/mod.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_app/src/runtime_presenter.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_normalizes_reassigned_page_table_truth_before_runtime_apply -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_reassigned_page_table_owner_in_next_frontier_recycle_plan -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
  - cargo test -p zircon_rhi --lib --tests
  - cargo test -p zircon_rhi_wgpu --lib --tests
  - cargo test -p zircon_render_graph --lib --tests
  - cargo test -p zircon_graphics --locked render_framework_bridge
  - cargo test -p zircon_scene --test render_frame_extract --locked
  - cargo test -p zircon_scene --locked
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features --locked
  - cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_consumes_feedback_and_promotes_requested_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_keeps_processing_later_unique_feedback_completions_after_leading_duplicate_requested_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_leaves_requests_pending_without_evictable_budget --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_respects_streaming_bytes_even_with_evictable_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_skips_oversized_requests_and_completes_ones_that_fit --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_assigns_free_slots_before_recycling_evictable_slots --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_streaming_state_changes_fallback_raster_output --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_streaming_state_changes_fallback_raster_coverage --locked
  - cargo test -p zircon_graphics virtual_geometry_submission_execution_order --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics visibility_context_builds_hybrid_gi_probe_and_trace_plan --locked
  - cargo test -p zircon_graphics visibility_context_with_history_tracks_hybrid_gi_requested_probes --locked
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_card_capture_samples_change_with_material_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_occupancy_changes_with_mesh_translation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_voxel_clipmap_occupancy_mask_moves_when_mesh_crosses_cells -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_requests_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_clipmaps_move_near_or_far_from_probe -- --nocapture
  - cargo check -p zircon_runtime --locked --lib
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_probe_or_trace_scene_data_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_builds_prepare_frame_without_host_bootstrap_irradiance --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_previous_irradiance_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_consumes_feedback_and_promotes_requested_probes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_leaves_updates_pending_without_evictable_budget --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_tracks_page_table_and_request_sink --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_deduplicates_requests_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_keeps_processing_later_valid_gpu_completions_after_leading_stale_slot_assignments --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_applies_gpu_assigned_free_slots_before_evictable_recycling --locked
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
  - cargo test -p zircon_graphics history_resolve_blends_previous_scene_color_when_enabled --locked
  - cargo test -p zircon_graphics ssao_quality_profile_darkens_scene_when_enabled --locked
  - cargo test -p zircon_graphics clustered_lighting_quality_profile_applies_runtime_tile_lighting --locked
  - cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics bloom_quality_profile_spreads_bright_pixels_when_enabled --locked
  - cargo test -p zircon_graphics color_grading_extract_tints_scene_after_post_process --locked
  - cargo test -p zircon_graphics offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering --locked
  - cargo test -p zircon_graphics particle_rendering_draws_billboard_sprites_in_transparent_stage --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_graphics render_server_tracks_viewports_and_accepts_frame_extract_submission
  - cargo test -p zircon_editor render_frame_extract_matches_legacy_render_snapshot_projection
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor slint_viewport_toolbar_pointer --locked
  - cargo test -p zircon_editor host::slint_host::viewport
  - cargo test -p zircon_editor tests::host::render_framework_boundary
  - cargo test -p zircon_app runtime_presenter
  - cargo test -p zircon_app --lib --locked
  - cargo test -p zircon_app runtime_sources_route_preview_through_render_framework_without_wgpu_surface_bindings
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b
  - cargo test -p zircon_graphics --lib project_render --locked --target-dir target/codex-shared-b
  - cargo check --workspace --locked --target-dir target/codex-shared-b
doc_type: module-detail
---

# Render Framework Architecture

## Purpose

这份文档记录本轮已经真正落地的渲染基础边界，而不是路线图里的全部长期目标。

当前交付集中在两个目标：

- 把渲染基础边界从单一 `zircon_graphics` 里切开，形成 `zircon_rhi`、`zircon_rhi_wgpu`、`zircon_render_graph`、`zircon_framework`
- 把场景渲染输入从旧 `RenderSceneSnapshot` 提升到新的 `RenderFrameExtract`，同时保留一个明确的兼容桥给现有 viewport 路径

## Landed Crate Roles

### `zircon_rhi`

当前承载无场景语义的底层图形接口类型：

- `RenderBackendCaps`
- `RenderQueueClass`
- `AccelerationStructureCaps`
- `BufferDesc` / `TextureDesc` / `SamplerDesc`
- `PipelineDesc` / `SwapchainDesc`
- `RenderDevice` / `CommandList`

这里故意不出现 mesh、material、light、particle、scene 这些上层概念。

### `zircon_rhi_wgpu`

当前不是完整设备后端，而是 `wgpu` 基线能力映射层：

- `wgpu_backend_caps(...)` 负责把当前 `wgpu` 基线映射到 `zircon_rhi::RenderBackendCaps`
- `WgpuRenderDevice` / `WgpuCommandList` 作为后续真正设备接入前的稳定包装

本轮明确保持 RT/AS 相关能力关闭，确保高级特性只能走 capability gate，而不是偷偷从 `wgpu` 类型向上泄漏。

### `zircon_render_graph`

当前落地的是可编译的 RenderGraph 骨架：

- `RenderGraphBuilder`
- `RenderPassId`
- `QueueLane`
- `PassFlags`
- `TransientTexture` / `TransientBuffer` / `ExternalResource`
- `CompiledRenderGraph`

它现在负责 pass 拓扑、依赖排序和 cycle rejection，还没有承担真正的命令录制与资源别名优化执行器。

### `zircon_framework`

这是新的稳定渲染 façade crate，当前提供：

- `RenderFramework`
- `RenderViewportHandle`
- `RenderPipelineHandle`
- `RenderViewportDescriptor`
- `RenderQualityProfile`
- `RenderStats`
- `CapturedFrame`
- `RenderCommand` / `RenderQuery`
- `RenderFrameworkHandle` / `resolve_render_framework(...)`

`RenderStats` 现在不仅承载 frame/view/pipeline 计数，还额外带一份 `RenderCapabilitySummary`：

- `backend_name`
- `queue_classes`
- `supports_surface` / `supports_offscreen`
- `supports_async_compute` / `supports_async_copy`
- `supports_pipeline_cache`
- `acceleration_structures_supported`
- `inline_ray_query`
- `ray_tracing_pipeline`
- `virtual_geometry_supported`
- `hybrid_global_illumination_supported`

这份摘要由 `zircon_graphics::runtime::WgpuRenderFramework` 从 `zircon_rhi_wgpu::WgpuRenderDevice` 基线能力映射出来，用来给后续 RT/GI/Virtual Geometry feature 做 façade 侧 capability gate，但不会把 `zircon_rhi` 或 `wgpu` 原生类型直接推给 editor/runtime/script。

`RenderQualityProfile` 当前也不再只是名字字符串。它已经支持：

- `pipeline_override`
- `RenderFeatureQualitySettings.clustered_lighting`
- `RenderFeatureQualitySettings.screen_space_ambient_occlusion`
- `RenderFeatureQualitySettings.history_resolve`
- `RenderFeatureQualitySettings.bloom`
- `RenderFeatureQualitySettings.color_grading`
- `RenderFeatureQualitySettings.reflection_probes`
- `RenderFeatureQualitySettings.baked_lighting`
- `RenderFeatureQualitySettings.particle_rendering`
- `RenderFeatureQualitySettings.virtual_geometry`
- `RenderFeatureQualitySettings.hybrid_global_illumination`
- `RenderFeatureQualitySettings.allow_async_compute`

这意味着 viewport 在没有显式 `set_pipeline_asset(...)` 时，既可以通过 quality profile 选择默认 built-in pipeline，也可以直接控制当前 M4 行为层里 `clustered lighting / SSAO / history / bloom / color grading / reflection probes / baked lighting / particle rendering` 的启闭与 async-compute 偏好；同时还可以对 `virtual geometry / hybrid global illumination` 发出 opt-in 请求，但这些旗舰路径只会在 backend capability 满足时进入有效编译结果。

`RenderStats` 当前还会带上 `last_frame_history`。`FrameHistoryHandle` 定义在 `zircon_framework` 的稳定 handle 层，并由 `zircon_graphics` 在 extract 子域重导出给 renderer/SRP 侧继续使用。这样 viewport history 生命周期既能被 façade 侧观测，又不会把 backend 私有资源类型推给上层。

为了让 behavior-layer 编译结果对 façade 可见，`RenderStats` 当前还会暴露：

- `last_effective_features`
- `last_async_compute_pass_count`
- `last_virtual_geometry_page_table_entry_count`
- `last_virtual_geometry_resident_page_count`
- `last_virtual_geometry_pending_request_count`
- `last_virtual_geometry_completed_page_count`
- `last_virtual_geometry_replaced_page_count`
- `last_virtual_geometry_indirect_draw_count`
- `last_virtual_geometry_indirect_args_count`
- `last_virtual_geometry_indirect_segment_count`
- `last_virtual_geometry_execution_segment_count`
- `last_virtual_geometry_execution_page_count`
- `last_virtual_geometry_execution_resident_segment_count`
- `last_virtual_geometry_execution_pending_segment_count`
- `last_virtual_geometry_execution_missing_segment_count`
- `last_virtual_geometry_execution_repeated_draw_count`
- `last_virtual_geometry_indirect_buffer_count`
- `last_hybrid_gi_cache_entry_count`
- `last_hybrid_gi_resident_probe_count`
- `last_hybrid_gi_pending_update_count`
- `last_hybrid_gi_scheduled_trace_region_count`

前两者用于验证 quality/capability 之后真正还留下了哪些内建 feature，以及 async-compute pass 是否已经 cleanly 降级到 graphics queue；中间这组 Virtual Geometry 字段用于同时观测两层真值：一层是 runtime host 当前持有的 page-table、resident page、pending request、completed/replaced page 规模，另一层是 renderer-local indirect raster 的 prepare-owned totals 与 actual execution subset totals。也就是说 façade 现在不只知道 shared indirect 是以多少 draw / args / segment / buffer 被构造出来，还能直接看到真实执行子集里到底有多少 unique segment/page、其中 resident/pending/missing fallback 各占多少，以及 repeated primitive compaction 在 execution 面真正压掉了多少 draw；最后四者用于观测 Hybrid GI runtime host 当前维护的 probe cache、resident probe、pending probe update 与 trace schedule 规模。

## Scene Extract Transition

`zircon_scene` 新增了 `render_extract.rs`，把新的提取面固定为：

- `RenderWorldSnapshotHandle`
- `RenderExtractContext`
- `RenderExtractProducer`
- `RenderViewExtract`
- `GeometryExtract`
- `LightingExtract`
- `PostProcessExtract`
- `DebugOverlayExtract`
- `ParticleExtract`
- `VisibilityInput`
- `RenderFrameExtract`

当前 `RenderFrameExtract` 仍然通过 `RenderFrameExtract::from_snapshot(...)` 和 `to_legacy_snapshot()` 与旧 `RenderSceneSnapshot` 双向适配。也就是说：

- 新边界已经存在，新的 render server 和后续 SRP 功能可以围绕它继续扩展
- 旧 `SceneRenderer`、`RenderService`、`RuntimePreviewRenderer` 还没有被删除，它们仍然通过 `ViewportRenderFrame` 中的兼容 `scene` 字段工作

这条双写桥是刻意保留的过渡层，不应被视为长期设计。

在 `M4` 的后半段，这个 extract 面已经继续长出真实行为层数据，而不是只保留空壳 section：

- `PostProcessExtract` 现在额外携带 `RenderBloomSettings` 与 `RenderColorGradingSettings`
- `LightingExtract` 现在额外携带 `reflection_probes` 与 `baked_lighting`
- `ParticleExtract` 现在额外携带 billboard 级 `sprites`
- `GeometryExtract` 现在额外预埋 `virtual_geometry: Option<RenderVirtualGeometryExtract>`
- `LightingExtract` 现在额外预埋 `hybrid_global_illumination: Option<RenderHybridGiExtract>`
- `zircon_scene::components` 公开了 `RenderReflectionProbeSnapshot`、`RenderBakedLightingExtract`、`RenderParticleSpriteSnapshot` 这组新的跨 crate snapshot 契约

这意味着后续 behavior layer 已经不需要偷偷回读旧 `RenderSceneSnapshot` 才能工作；新增后处理、烘焙和粒子路径都已经可以只消费 `RenderFrameExtract`。

## `zircon_graphics` Current Shape

`zircon_graphics` 现在开始向高层 SRP/Renderer crate 收束，新增了固定子域：

- `compat/`
- `extract/`
- `feature/`
- `material/`
- `pipeline/`
- `runtime/`
- `shader/`
- `visibility/`

本轮真正有行为的新增核心是 `runtime::WgpuRenderFramework`：

- 通过 `RenderFramework` trait 暴露创建 viewport、提交 `RenderFrameExtract`、设置 pipeline/quality、统计与 capture 的稳定接口
- 内部暂时仍复用现有 `SceneRenderer` 做离屏渲染
- `GraphicsModule.Manager.RenderFramework` 现在由 `zircon_runtime::graphics::module_descriptor()` 注册；`zircon_graphics` 只保留底层 renderer / framework 实现，不再持有 module-host surface

同时保留了旧兼容面：

- `RenderService`
- `SharedTextureRenderService`
- `RuntimePreviewRenderer`

这三者现在主要是 `zircon_graphics` 内部兼容能力，不再是 editor/runtime 的主消费路径。

## Consumer State

当前 consumer 已经切到 `RenderFramework` façade：

- `zircon_editor::host::slint_host::viewport::SlintViewportController` 在初始化时直接 `resolve_render_framework(...)`
- `EditorState` / `EditorEventRuntime` 现在直接暴露 `render_frame_extract()`，把旧 `RenderSceneSnapshot` 适配压回状态层
- editor viewport 不再走 shared texture + `wgpu` 导入，而是提交 `RenderFrameExtract`，然后把 `CapturedFrame.rgba` 转成 `slint::Image`
- `zircon_app::runtime_presenter::RenderFrameworkRuntimeBridge` 负责 runtime viewport handle 生命周期、frame submit 与 capture
- runtime 入口自身通过 `World::to_render_frame_extract()` 直接生成 extract，不再在 `lib.rs` 里手写 snapshot 兼容适配
- runtime window 不再依赖 `RuntimePreviewRenderer` 的 `wgpu` surface path，而是使用 `SoftbufferRuntimePresenter` 把 `RenderFramework` 输出的 RGBA 帧 blit 到窗口
- `runtime_bootstrap_excludes_editor_module` 现在显式验证 runtime bootstrap 后可以 `resolve_render_framework(&core)`
- editor/runtime 额外有源码边界守卫测试，防止后续重新引入 `wgpu`、shared-texture preview renderer 或旧 preview façade

当前仍未完成的迁移包括：

- `zircon_manager::RenderingManager` 退化成纯兼容桥
- `zircon_graphics` compat 层对旧 `RenderService` / `RuntimePreviewRenderer` 的最终收束与删除
- shader/material hot reload、真正的 feature 实例注册和 GPU-driven visibility 前处理

## Default SRP Compile Skeleton

`zircon_graphics::pipeline` 现在已经不再只靠 `stage -> pass name` 的硬编码表编译默认 Forward+。

当前固定的 M2 编译骨架是：

- `BuiltinRenderFeature` 负责声明 feature 级 extract 依赖和 stage pass 贡献
- `RenderPipelineAsset::compile(...)` 负责验证 `RendererAsset`，收集启用 feature 的 descriptor，然后按 renderer stage 顺序把 pass 编译进 `CompiledRenderGraph`
- `runtime::WgpuRenderFramework` 只消费编译结果，不在 submit 时重新推断 stage/pass 结构

当前内建的默认 Forward+ feature 组合包括：

- `VirtualGeometry`
- `Mesh`
- `Shadows`
- `ScreenSpaceAmbientOcclusion`
- `ClusteredLighting`
- `GlobalIllumination`
- `Particle`
- `Bloom`
- `ReflectionProbes`
- `BakedLighting`
- `PostProcess`
- `ColorGrading`
- `HistoryResolve`
- `DebugOverlay`

其中 `VirtualGeometry` 与 `GlobalIllumination` 当前属于 “声明在 built-in renderer 里，但默认不进入有效编译结果” 的旗舰槽位。它们只有在 compile options 显式 opt-in 时才会真正贡献 pass 和 history binding。

这带来三个关键边界改进：

- overlay/gizmo 不再通过 `PostProcess` 间接声明 `debug` extract 依赖，而是由独立 `DebugOverlay` feature 负责
- pipeline compile 会显式拒绝 duplicate stage 和 duplicate feature 配置，而不是把不确定顺序静默吞掉
- `VirtualGeometry / GlobalIllumination / RayTracing` 这类旗舰 feature 现在可以沿同一套 descriptor 系统挂进 pipeline，但不会反向污染基础 renderer 默认结果

当前 pipeline compile 也已经进入 M4 的“可配置编译”阶段。`RenderPipelineAsset` 除了默认 `compile(...)`，现在还支持 `compile_with_options(...)`，并消费 `RenderPipelineCompileOptions`：

- `enabled_features`
- `disabled_features`
- `allow_async_compute`

这让 built-in pipeline 可以在不改动资产结构的前提下，根据 quality profile 和 capability 生成不同的有效 feature 集合与 queue-lane 分布；同时也让 M5 的旗舰路径第一次拥有显式 opt-in 编译入口，而不是默认随 built-in pipeline 一起混入。

对当前默认 pipeline，编译后的固定阶段仍然保持为：

- `DepthPrepass`
- `Shadow`
- `Opaque`
- `Transparent`
- `PostProcess`
- `Overlay`

但 graph pass 来源已经迁移到 feature descriptor 层，后续继续扩展 `Particle`、`Deferred`、`GI`、`Virtual Geometry` 时，不需要再把特殊分支塞回中心硬编码点。

## M4 Deferred Pipeline Runtime

`M4` 的第二条内建 renderer/pipeline 不再只是 compile skeleton。当前 `RenderPipelineAsset`、`RenderFramework`、`SceneRenderer` 已经共同落下一条真实 deferred runtime：opaque 几何不再复用 forward project shader，而是固定改走 GBuffer 材质解码和 fullscreen deferred lighting。

当前已经固定下来的 deferred pipeline 边界是：

- `RenderPassStage` 新增 `GBuffer` 与 `Lighting`
- `BuiltinRenderFeature` 新增 `DeferredGeometry` 与 `DeferredLighting`
- `RenderPipelineAsset::default_deferred()` 现在固定占用 built-in handle `2`
- `RenderPipelineAsset::builtin(...)` 现在同时能解析 built-in Forward+ 与 built-in Deferred
- `runtime::WgpuRenderFramework` 的 built-in pipeline registry 现在同时注册两条内建 pipeline，因此 viewport 可以显式切到 deferred handle 再走完整 submit/capture 路径
- `RenderQualityProfile::pipeline_override` 现在也能把 deferred pipeline 作为 viewport 的默认 renderer 选择源，而不要求 consumer 先手动 `set_pipeline_asset(...)`

当前 deferred pipeline 的固定阶段顺序是：

- `DepthPrepass`
- `Shadow`
- `GBuffer`
- `AmbientOcclusion`
- `Lighting`
- `Transparent`
- `PostProcess`
- `Overlay`

当前 deferred pipeline 的 built-in feature 组合是：

- `DeferredGeometry`
- `Shadows`
- `ScreenSpaceAmbientOcclusion`
- `ClusteredLighting`
- `DeferredLighting`
- `Particle`
- `Bloom`
- `ReflectionProbes`
- `BakedLighting`
- `PostProcess`
- `ColorGrading`
- `HistoryResolve`
- `DebugOverlay`

编译后当前固定 pass 顺序是：

- `depth-prepass`
- `shadow-map`
- `gbuffer-mesh`
- `ssao-evaluate`
- `clustered-light-culling`
- `deferred-lighting`
- `transparent-mesh`
- `particle-render`
- `bloom-extract`
- `reflection-probe-composite`
- `baked-lighting-composite`
- `post-process`
- `color-grade`
- `history-resolve`
- `overlay-gizmo`

这条实现故意保持“先固定 pipeline/feature graph，再补真正 shading path”的边界：

- `SceneRendererCore` 现在会按 `CompiledRenderPipeline.enabled_features` 真实分支 forward 与 deferred runtime，而不是只在 compile graph 层区分
- `OffscreenTarget` 当前新增 `gbuffer_albedo`，并继续复用 `normal / depth / scene_color / final_color / ambient_occlusion / cluster_buffer`
- `ViewportOverlayRenderer` 现在把 scene content 进一步拆成 `record_preview_sky(...)` 与 `record_meshes(...)`，这样 deferred 可以只消费背景与透明补绘，而不会重新掉回整条 forward base-scene pass

### Real Deferred Runtime Path

当前 built-in deferred 的真实执行顺序已经固定为：

- `record_preview_sky(...)` 先把 clear color / preview sky 写入 `final_color`，作为 deferred lighting 的背景输入
- `NormalPrepassPipeline` 只对 opaque draw 写入 `normal + depth`
- `DeferredSceneResources::record_gbuffer_geometry(...)` 把 opaque draw 的 `albedo texture * material tint` 写入 `gbuffer_albedo`
- `ScenePostProcessResources::execute_ssao(...)` 与 `execute_clustered_lighting(...)` 继续跑在共享的 `normal / depth / cluster_buffer` 上
- `DeferredSceneResources::execute_lighting(...)` 读取 `gbuffer_albedo + normal + final_color(background)`，输出 lit opaque scene 到 `scene_color`
- `ViewportOverlayRenderer::record_meshes(...)` 再把 transparent draw 用现有 forward mesh path 叠加进 `scene_color`
- `ScenePostProcessResources::execute_post_process(...)` 继续复用既有 `scene_color + AO + history + cluster_buffer -> final_color` 链
- `ViewportOverlayRenderer::record_overlays(...)` 最后把 gizmo/selection/handle 叠加到 `final_color`

这条实现当前刻意保持以下边界：

- opaque deferred geometry 只做稳定材质解码，不执行项目自定义 fragment shader
- transparent 仍走现有 forward mesh path，避免第一轮 deferred baseline 把透明材质和排序语义一并重写
- clustered lighting / SSAO / history resolve 仍然通过共享 post/runtime 资源链生效，而不是在 deferred 路径里私有复制一套
- deferred 目前的 GBuffer 只先落 `albedo`，normal 继续复用已有 normal prepass，足够支撑当前 baseline 的 material decode + directional/ambient lighting

## M4 Clustered Lighting SSAO History Baseline

当前已经把 `clustered lighting / SSAO / history` 这一批 M4 行为层推进到“真实 shader/resource/runtime path”阶段。它们不再只是 compile skeleton，而是由 `WgpuRenderFramework -> SceneRenderer` 真正驱动 GPU 资源、compute/fullscreen pass 和跨帧 history copy。

### Frame History Contract

`zircon_graphics::extract` 当前已经把跨帧资源 contract 固定成：

- `FrameHistorySlot`
  - `AmbientOcclusion`
  - `SceneColor`
- `FrameHistoryAccess`
  - `Read`
  - `Write`
  - `ReadWrite`
- `FrameHistoryBinding`

`RenderFeatureDescriptor` 现在除了 `required_extract_sections`，还会显式声明 `history_bindings`。`RenderPipelineAsset::compile(...)` 会按 slot 聚合 enabled feature 的 history usage，并把结果输出到 `CompiledRenderPipeline.history_bindings`。当前 merge 规则固定为：

- 同 slot 的 `Read + Write` 会折叠成 `ReadWrite`
- 任一侧已是 `ReadWrite` 时保持 `ReadWrite`
- 输出顺序按 `FrameHistorySlot` 稳定排序

### Built-In Feature Wiring

当前新增的 built-in feature 族包括：

- `ClusteredLighting`
- `ScreenSpaceAmbientOcclusion`
- `HistoryResolve`

同时 `RenderPassStage` 新增了 `AmbientOcclusion`，让 built-in pipeline 现在可以显式编译：

- `ssao-evaluate`
- `clustered-light-culling`
- `history-resolve`

Forward+ 当前固定阶段顺序变成：

- `DepthPrepass`
- `Shadow`
- `AmbientOcclusion`
- `Lighting`
- `Opaque`
- `Transparent`
- `PostProcess`
- `Overlay`

Deferred 当前固定阶段顺序则变成：

- `DepthPrepass`
- `Shadow`
- `GBuffer`
- `AmbientOcclusion`
- `Lighting`
- `Transparent`
- `PostProcess`
- `Overlay`

compile 边界现在仍然保持稳定，但这些 feature 不再只是空声明：

- `ClusteredLighting` 只声明 `clustered-light-culling` async compute pass 和它需要的 extract section
- `ScreenSpaceAmbientOcclusion` 只声明 `ssao-evaluate` async compute pass，并把 `AmbientOcclusion` slot 标成 `ReadWrite`
- `HistoryResolve` 只声明 `history-resolve` graphics pass，并把 `SceneColor` slot 标成 `ReadWrite`

在这之上，当前已经补上两条真正的行为层闭环：

- `RenderPipelineCompileOptions` 可以显式禁用这三个 built-in feature
- 当 `allow_async_compute = false` 时，这两条 compute pass 会 cleanly 退化到 graphics queue，而不是继续输出 `AsyncCompute`

### Real Runtime Resource Path

`SceneRenderer` 现在新增了一条 server-only runtime 入口：`render_frame_with_pipeline(...)`。`WgpuRenderFramework` 在 submit 时不再只是编译 pipeline 然后继续走旧单 target render，而是把 `CompiledRenderPipeline + FrameHistoryHandle` 直接交给 renderer 执行。

当前真实落地的资源链是：

- `OffscreenTarget.final_color`：最终 readback / capture 的颜色目标
- `OffscreenTarget.scene_color`：base scene 先写入的中间 scene color
- `OffscreenTarget.normal`：normal/depth prepass 输出的法线缓冲
- `OffscreenTarget.ambient_occlusion`：SSAO compute pass 输出的 AO 纹理
- `OffscreenTarget.depth`：现在是可采样 depth texture，而不是只做 render attachment
- `OffscreenTarget.cluster_buffer`：按 tile 写入的 clustered-light runtime buffer

对应的 runtime pass 链现在是：

- `NormalPrepassPipeline`：先把世界法线写进 `normal`，并建立可采样 depth
- `ViewportOverlayRenderer::record_scene_content(...)`：把 preview sky + base mesh scene 写进 `scene_color`
- `ScenePostProcessResources::execute_ssao(...)`：读取 `depth + normal + previous AO history`，写入 `ambient_occlusion`
- `ScenePostProcessResources::execute_clustered_lighting(...)`：把 extract lighting 编码进 GPU light buffer，并按 tile 写进 `cluster_buffer`
- `ScenePostProcessResources::execute_post_process(...)`：读取 `scene_color + ambient_occlusion + previous scene color history + cluster_buffer`，输出 `final_color`
- `ViewportOverlayRenderer::record_overlays(...)`：最后把 wire/selection/gizmo/handle 叠加回 `final_color`

这一条实现故意仍然保持“真正执行 M4 行为层，但不假装已经完成完整 deferred shading”的边界：

- clustered lighting 目前是基于 directional light extract 的 tile lighting buffer，而不是完整 local-light clustered shading
- SSAO 当前是基于 depth/normal 的 compute AO，并且先把 AO 作为 post composite 输入，而不是接入完整材质解码链
- history resolve 当前先落 `scene color` 与 `ambient occlusion` 两条跨帧 history copy，而不是直接宣称 TAA 完成

### Viewport Runtime History Host

`zircon_graphics::runtime` 当前新增了 `ViewportFrameHistory`，由 `WgpuRenderFramework` 为每个 viewport 持有：

- `handle`
- `viewport_size`
- `pipeline`
- `generation`
- `bindings`
- `visibility`

`submit_frame_extract(...)` 现在会在 render 前读取上一帧 visibility history，调用 `VisibilityContext::from_extract_with_history(...)` 构建统一前处理的跨帧 diff 输入。render 完成后则按下面的规则决定复用还是轮换 `FrameHistoryHandle`：

- viewport 还没有 history 时分配新 handle
- `viewport_size` 变化时轮换
- `pipeline` 变化时轮换
- `history_bindings` 变化时轮换
- 兼容时只更新 `generation / bindings / visibility`

renderer 侧则由 `SceneFrameHistoryTextures` 真正承载 GPU history 资源：

- `scene_color_history`
- `ambient_occlusion_history`

这些纹理按 `FrameHistoryHandle` 建立和回收。viewport destroy 或 handle 轮换时，`WgpuRenderFramework` 会同步调用 renderer 的 `release_history(...)`，而不是让旧 history texture 无限堆积。

这意味着 `clustered lighting / SSAO / history resolve` 现在已经不再只是 pipeline compile-time 元数据，而是正式拥有了 per-viewport runtime 宿主和真实跨帧 texture copy 行为。

`WgpuRenderFramework` 当前还会把 `RenderQualityProfile + RenderCapabilitySummary` 映射成 `RenderPipelineCompileOptions`：

- quality profile 可以禁用 `clustered lighting / SSAO / history resolve`
- `allow_async_compute` 需要同时满足 profile 允许和 backend capability 支持
- 当前 headless `wgpu` 基线 `supports_async_compute = false`，因此默认 built-in pipeline 会保留 feature，但把 `ssao-evaluate` 和 `clustered-light-culling` cleanly 降级到 graphics queue
- `RenderStats.last_effective_features` 与 `last_async_compute_pass_count` 会记录这一结果

## M4 Remaining Behavior Layers Baseline

`M4` 剩余的行为层当前已经不再停留在 pass/profile skeleton，而是进入真正的 shader/resource/runtime 数据路径：

- `OffscreenTarget` 现在新增 `bloom` 中间纹理，`scene color -> bloom -> final color` 这条后处理链已经有独立资源位
- `ScenePostProcessResources` 现在额外持有：
  - bloom fullscreen pipeline
  - bloom params uniform
  - reflection probe storage buffer
  - 扩展后的 post-process uniform/bind group layout
- `post_process.wgsl` 不再只做 `AO + clustered + history` composite，而是继续消费：
  - bloom 纹理
  - color grading 参数
  - projected reflection probe buffer
  - baked lighting 参数
- `RenderPipelineCompileOptions` 与 `RenderQualityProfile` 现在会真正影响这些行为层的内建 feature 集，而不只是 façade 侧挂名开关

### Bloom And Color Grading Runtime

当前 bloom 不是 fake CPU 后处理，而是 renderer 内的真实 fullscreen pass：

- `execute_bloom(...)` 先从 `scene_color` 里提取超过阈值的亮部，并做一个软化采样
- 最终 `execute_post_process(...)` 再把 bloom 纹理叠回主颜色
- color grading 使用 extract 提供的 `exposure / contrast / saturation / gamma / tint` 参数，在最终 composite shader 内统一执行

这条边界刻意保持简单：

- 不引入复杂 mip-chain 或多级 downsample ping-pong
- 不把 bloom/color grading 私有化进某个 pipeline 分支
- 仍然继续走 `RenderFramework -> CompiledRenderPipeline -> SceneRenderer` 的统一 runtime 路径

### Reflection Probes And Baked Lighting Runtime

reflection probes 与 baked lighting 当前已经有一条 capability/profile gated 的真实 baseline：

- `offline_bake_frame(...)` 会从 `RenderFrameExtract` 的方向光和几何体提取里生成：
  - `RenderBakedLightingExtract`
  - `Vec<RenderReflectionProbeSnapshot>`
- renderer 在运行时会把 probe 数据编码成 projected screen-space influence buffer，而不是在 shader 里重新访问 scene/world
- 最终 composite 会把 probe 贡献和 baked ambient 统一叠进主颜色

这条实现仍然刻意是 baseline，而不是伪装成完整 probe/GI 系统：

- probe 目前是 screen-space projected influence，不是 cubemap capture + parallax correction
- baked lighting 目前是 CPU 端 baseline 输出，不是完整 lightmap/irradiance volume 流程
- 但 scene/extract/runtime/profile 边界已经预埋到位，后续继续换成更强实现时不需要回退到旧 snapshot 或 backend 私有路径

### Particle Transparent Stage Runtime

粒子当前已经有独立的透明阶段 runtime pass，而不是 overlay hack：

- `ParticleRenderer` 会从 `RenderParticleSpriteSnapshot` 组装 CPU 侧 billboard 顶点流
- pass 在 scene color 的 transparent 阶段执行，并使用 additive color blend
- `particle_rendering` quality toggle 会通过 built-in feature disable path 干净关闭整条粒子 pass

这让粒子在架构上继续归属于统一 SRP feature/pipeline/runtime 体系，而不是重新开一条私有 renderer 分支。

## M5 Flagship Capability Slots Baseline

`M5` 当前先落的是 “flagship feature capability slot”，不是 Nanite/Lumen 本体。目标是把 `Virtual Geometry` 与 `Hybrid GI` 变成架构上真实存在、但默认关闭、并且完全 capability-gated 的 `RenderFeature` 家族。

当前已经固定下来的边界是：

- `zircon_framework::render::GeometryExtract` 新增 `virtual_geometry: Option<RenderVirtualGeometryExtract>`，默认由 legacy snapshot adapter 初始化为 `None`
- `zircon_framework::render::LightingExtract` 新增 `hybrid_global_illumination: Option<RenderHybridGiExtract>`，默认同样保持 `None`
- `zircon_framework::RenderFeatureQualitySettings` 新增 `virtual_geometry` 与 `hybrid_global_illumination` 两个 profile 开关，默认值都为 `false`
- `RenderCapabilitySummary` 新增 `virtual_geometry_supported` 与 `hybrid_global_illumination_supported`，作为 façade 可观测的 backend capability 摘要
- `FrameHistorySlot` 新增 `GlobalIllumination`
- `BuiltinRenderFeature::VirtualGeometry` 现在会在 opt-in 时贡献 `virtual-geometry-prepare`
- `BuiltinRenderFeature::GlobalIllumination` 现在会在 opt-in 时贡献 `hybrid-gi-resolve`，并把 `GlobalIllumination` history slot 标记为 `ReadWrite`

这一轮最重要的 compile 规则是：

- 默认 built-in Forward+ / Deferred renderer 仍然保留 `VirtualGeometry` 与 `GlobalIllumination` 的 descriptor 槽位，但 compile 时不会自动启用它们
- `RenderPipelineCompileOptions::with_feature_enabled(...)` 负责显式 opt-in；没有 opt-in 时，默认 pass 顺序与 M4 完全保持不变
- `requires_explicit_opt_in()` 当前把 `VirtualGeometry / GlobalIllumination / RayTracing` 收进同一条旗舰功能门控规则，避免后续高阶路径重新把默认 pipeline 污染回基础层

`WgpuRenderFramework` 当前把 façade 层 profile 与 backend capability 映射成有效 compile options：

- `virtual_geometry_supported` 目前要求 `supports_async_compute && supports_pipeline_cache`
- `hybrid_global_illumination_supported` 目前要求 `acceleration_structures_supported && (inline_ray_query || ray_tracing_pipeline)`
- 因为当前 headless `wgpu` 基线不满足这些条件，所以即使 quality profile 显式请求 `virtual_geometry / hybrid_global_illumination`，`last_effective_features` 里也不会出现它们

这条实现刻意保持边界预埋而不是伪装旗舰技术已经完整：

- 已经补上 renderer-local 的最小 GPU uploader/readback、`size_bytes` 驱动的 streaming-byte arbitration、hierarchy refine、cluster-streaming fallback consumption、indirect raster baseline，以及 Hybrid GI 的 trace/update/irradiance readback、temporal radiance-cache update、trace-region `rt_lighting_rgb` override、screen-probe parent-child hierarchy baseline 与 radiance-cache lighting resolve，但还没有 Nanite/Lumen-like 真实场景表示、GPU-driven indirect compaction 或完整 RT hybrid lighting
- 但 extract、profile、history、pipeline compile、runtime capability gate、stats 可观测性已经对齐，后续继续实现真实旗舰路径时不需要重新拆 façade 边界

### M5 Virtual Geometry Preprocess Baseline

在 capability-slot 之上，当前又把 `Virtual Geometry` 推进了一层统一前处理 baseline，但仍然刻意停在 “数据规划” 而不是 “真实执行器”。

当前新增的 scene/extract 合同是：

- `RenderVirtualGeometryExtract`
  - `cluster_budget`
  - `page_budget`
  - `clusters`
  - `pages`
- `RenderVirtualGeometryCluster`
  - `entity`
  - `cluster_id`
  - `page_id`
  - `lod_level`
  - `bounds_center`
  - `bounds_radius`
  - `screen_space_error`
- `RenderVirtualGeometryPage`
  - `page_id`
  - `resident`
  - `size_bytes`

`VisibilityContext` 当前新增三组 Virtual Geometry 前处理输出：

- `virtual_geometry_visible_clusters`
- `virtual_geometry_page_upload_plan`
- `virtual_geometry_feedback`

其中 page/upload/feedback 边界被固定成：

- `VisibilityVirtualGeometryPageUploadPlan.resident_pages`
- `VisibilityVirtualGeometryPageUploadPlan.requested_pages`
- `VisibilityVirtualGeometryPageUploadPlan.dirty_requested_pages`
- `VisibilityVirtualGeometryPageUploadPlan.evictable_pages`
- `VisibilityVirtualGeometryFeedback.visible_cluster_ids`
- `VisibilityVirtualGeometryFeedback.requested_pages`
- `VisibilityVirtualGeometryFeedback.evictable_pages`

这一层的规则当前是：

- Virtual Geometry cluster 不只跟随 entity 级 mesh visibility；它们会再走一次 cluster sphere frustum test
- visible cluster 排序按 `screen_space_error` 优先，然后按 `lod_level`、`cluster_id` 做稳定 tie-break
- `cluster_budget` 会截断本帧真正进入可见集的 cluster
- `page_budget` 会截断本帧真正发起请求的 missing page
- hierarchy refine 现在显式拆成 `streaming_target_clusters` 与 resident-gated `visible_clusters`：request 可以继续追更细 cluster，但 render 只有在 replacement children page 全部 resident 时才真正下沉
- `VisibilityHistorySnapshot` 当前额外保留 `virtual_geometry_requested_pages`，让 `dirty_requested_pages` 可以按跨帧 diff 计算，而不是每帧全量重发
- resident 但本帧不再属于 resident-gated render frontier 的页会进入 `evictable_pages`，因此 coarse parent 在 finer children/grandchildren 还没 resident 时不会被过早回收

render-server façade 当前也开始把这条前处理链的规模暴露到 `RenderStats`：

- `last_virtual_geometry_visible_cluster_count`
- `last_virtual_geometry_requested_page_count`
- `last_virtual_geometry_dirty_page_count`
- `last_virtual_geometry_page_table_entry_count`
- `last_virtual_geometry_resident_page_count`
- `last_virtual_geometry_pending_request_count`
- `last_virtual_geometry_completed_page_count`
- `last_virtual_geometry_replaced_page_count`
- `last_virtual_geometry_indirect_draw_count`
- `last_virtual_geometry_indirect_segment_count`

当前这些计数只有在 `VirtualGeometry` 真正进入有效 compiled pipeline 时才会写入。当前 pure `wgpu` headless baseline 已经会把这条非 RT M5 baseline 映射为 capability-supported，因此在 profile 显式 opt-in 且 extract 提供 payload 时，这些值会进入真实统计链；其中 `completed_page_count / replaced_page_count` 直接来自 GPU uploader completion 与 explicit replacement readback，而 `indirect_draw_count / indirect_segment_count` 则直接来自 prepare-owned unified indirect authority；如果 feature 没被请求或 extract 为空，它们仍然稳定保持 `0`。

### M5 Virtual Geometry Runtime Host Baseline

在 preprocess 之上，当前先补了 CPU 侧 runtime host 来承接 viewport 级的 page table、resident page 与 pending request 状态，随后再把最小 GPU uploader/readback 接进 renderer；目前仍然没有更完整的 streaming residency manager 或 cluster hierarchy。

当前已经固定下来的 runtime host 边界是：

- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 现在作为 viewport 级宿主，持有：
  - resident page -> slot 映射
  - resident page budget
  - page `size_bytes` metadata
  - pending request 队列
  - 当前帧 `evictable_pages`
- `ViewportRecord` 新增 `virtual_geometry_runtime`
- `WgpuRenderFramework::submit_frame_extract(...)` 现在会在 `BuiltinRenderFeature::VirtualGeometry` 真正进入有效 compiled pipeline 时：
  - 先用 `register_extract(...)` 同步 extract page metadata 与 resident baseline
  - 再用 `ingest_plan(...)` 吞入 visibility 生成的 resident/requested/dirty/evictable page 计划
  - 最后把 runtime host 规模写回 façade stats

这一层的规则当前是：

- resident baseline page 会保留稳定 slot，不会在重复提交时重新编号
- `RenderVirtualGeometryExtract.page_budget` 当前会同时约束 CPU fallback runtime host 的 resident page budget，并且至少覆盖 extract 自身声明的 resident baseline page
- `dirty_requested_pages` 只会转成一次 pending request，不会因重复提交而重复入队
- `apply_evictions(...)` 释放出来的 slot 会被后续 fulfill/reload 按最小空闲 slot 优先复用
- headless `wgpu` 基线依旧不会创建有效 runtime host，因为 capability gate 会让 `VirtualGeometry` feature 继续保持关闭

render-server façade 当前还额外暴露这三项 runtime host 计数：

- `last_virtual_geometry_page_table_entry_count`
- `last_virtual_geometry_resident_page_count`
- `last_virtual_geometry_pending_request_count`

它们与前处理计数一样，只会在 `VirtualGeometry` 真正进入有效 compiled pipeline 时写入；在当前 headless `wgpu` 基线上仍然稳定为 `0`。

### M5 Virtual Geometry Prepare Consumption Baseline

在 runtime host 之上，当前又把 `Virtual Geometry` 推进了一层真正进入 frame/runtime path 的 baseline：`virtual-geometry-prepare` 不再只是 compile-time pass 名字，而是已经会消费 viewport 级 runtime host，生成 frame-local prepare snapshot，并驱动当前 mesh fallback path 的实体过滤与部分栅格消费。

当前新增的 frame/runtime 合同是：

- graphics-internal `ViewportRenderFrame` 新增 `virtual_geometry_prepare` 槽位
- `VirtualGeometryPrepareFrame`
  - `visible_entities`
  - `visible_clusters`
  - `cluster_draw_segments`
  - `resident_pages`
  - `pending_page_requests`
  - `evictable_pages`
- `VirtualGeometryPrepareCluster`
  - `entity`
  - `cluster_id`
  - `page_id`
  - `lod_level`
  - `resident_slot`
  - `state: Resident | PendingUpload | Missing`
- `VirtualGeometryPrepareDrawSegment`
  - `entity`
  - `cluster_id`
  - `cluster_ordinal`
  - `cluster_count`
  - `lod_level`
  - `state: Resident | PendingUpload | Missing`

这一层的规则当前是：

- `VirtualGeometryRuntimeState::build_prepare_frame(...)` 会把 runtime host 当前 resident slot、pending request、evictable page 与本帧 visible cluster 合成 prepare snapshot，并且只为 `Resident` / `PendingUpload` cluster 生成显式 `cluster_draw_segments`
- `VirtualGeometryPrepareFrame::unified_indirect_draws()` 现在还会把 `pending_page_requests.assigned_slot / recycled_page_id` 与当前 resident page-table snapshot 收束成显式 `submission_slot`，让 fallback recycle-slot authority 继续进入 unified indirect submission，而不再只停在 uploader fallback path
- `VirtualGeometryPrepareFrame::unified_indirect_draws()` 现在还会为 “`visible_entities` 里可见、但当前帧缺少显式 `cluster_draw_segments`” 的 entity 直接合成 per-cluster fallback indirect draws；这些 fallback slices 会沿 `prepare -> build_virtual_geometry_cluster_raster_draws(...) -> draw_ref / indirect args / submission` 主链继续下沉，而不再主要依赖 renderer 末端的 CPU fallback side-path
- `build_mesh_draws(...)` 与 `extend_pending_draws_for_mesh_instance(...)` 现在也不再平行维护 `authoritative_fallback_segment_keys` 一类的 fallback bookkeeping；authoritative segment / draw-ref / pending-draw source 已经明确收敛到 `virtual_geometry_cluster_draws`，而且 `virtual_geometry_cluster_draws == None` 现在会被视为 authoritative no-draw truth，explicit `Missing` segment entity 不会再在 renderer 末端被复活成 CPU full-mesh fallback draw
- `runtime/virtual_geometry/{prepare_frame,pending_completion,residency_management}/`、`runtime/hybrid_gi/{prepare_frame,pending_completion,residency_management}/` 与 `runtime/render_framework/submit_frame_extract/{build_frame_submission_context,prepare_runtime_submission,submit,record_submission,update_stats}/` 现在都已经拆成结构入口 + helper 子模块，prepare snapshot、completion 回写、submit context/runtime prepare/submit/record/stats 汇总与 slot bookkeeping 不再堆在单个脚本里
- `build_virtual_geometry_plan(...)` 会在 visibility/preprocess 阶段为每个可见 cluster 计算稳定的 `cluster_ordinal / cluster_count`，而且这个 ordinal 固定从 entity 的完整 extract cluster 集导出，而不是只从本帧 frontier 导出
- 只有 `Resident` 或 `PendingUpload` cluster 对应的 entity 会进入 `visible_entities`；完全 `Missing` 的 page/cluster 会继续保留在 prepare snapshot 里，但不会进入当前 fallback draw 白名单
- `WgpuRenderFramework::submit_frame_extract(...)` 现在会在 render 之前就克隆并更新 viewport 级 runtime host，再把 prepare snapshot 挂到 graphics-internal `ViewportRenderFrame`
- `build_mesh_draws(...)` 现在会在 `VirtualGeometry` feature 显式开启时，使用 prepare snapshot 的 `visible_entities` 过滤当前 mesh fallback draw 集；显式 prepare segments 与 unified-indirect synthesized fallback slices 都会优先走同一份 cluster-raster draw truth，renderer 末端只保留更窄的兜底 fallback
- `BaseScenePass`、`NormalPrepassPipeline` 与 deferred geometry pass 都会消费这个 `first_index + draw_index_count`，因此 Resident/Pending cluster 状态现在会改变真实提交到 GPU 的 index 范围，而不再只是 tint 提示
- renderer 不再允许从 `extract.geometry.virtual_geometry.clusters` 反推 fallback cluster slice；prepare 的 `cluster_draw_segments` 是唯一 segment 合同
- renderer last-state 现在还会直接保留并回读真实 GPU-submitted indirect segment buffer，因此 unified-indirect 回归可以直接验证 submission segment truth，而不再只验证 prepare projection 和最终 indirect args
- 当 dedicated `submission_buffer` 与 renderer-local token record 都不可用时，last-state 现在还会回退直接解析真实 indirect args buffer 的 `first_instance` token；因此 draw-level submission observability 已经收敛到 actual GPU-generated args source，而不再只能依赖平行 debug side-channel

这条实现现在已经从纯 CPU fallback 推进到最小 GPU uploader/readback baseline，但仍然没有把更完整的 streaming ownership 做完：

- 当前已有最小 GPU page upload/readback baseline，但还没有 async copy queue、page residency manager 或 cluster-streaming ownership
- 当前也没有 GPU-driven indirect command compaction、cluster hierarchy split-merge、Nanite raster 或更深层的 visibility-owned indirect draw integration
- 但 `virtual-geometry-prepare` 已经拿到了 runtime host 的消费边界，后续 GPU uploader/streaming/refine 可以在不重拆 render path 的前提下继续向下替换

### M5 Virtual Geometry Feedback Streaming Baseline

在 prepare consumption 之后，当前又把此前未消费的 `VisibilityVirtualGeometryFeedback` 接进了 viewport runtime host。这样 pending page request 不再只是停在 request sink 统计里，而会在帧后按 resident budget 与 evictable resident page 列表推进到下一帧 residency。

这一层当前新增的合同是：

- `VirtualGeometryRuntimeState::consume_feedback(&VisibilityVirtualGeometryFeedback)`
- `WgpuRenderFramework::submit_frame_extract(...)` 现在会在 render 完成后消费当前帧 feedback，再把更新后的 runtime host 写回 viewport record
- façade stats 使用的是 feedback 消费之后的 runtime snapshot，因此当 capability 未来放开时，`page_table / resident / pending-request` 规模会反映 submit 完成后的宿主状态，而不是 render 前的中间状态

这一层的规则当前是：

- 只有当前 feedback 中仍然处于 pending 的 `requested_pages` 才会被尝试 promote
- `build_prepare_frame(...)` 导出的 pending uploader queue 现在会先按当前 visibility 提供的 `requested_pages` frontier 顺序排序，只有当前 request rank 打平时才回退到 hot-descendant / hierarchy depth 启发式
- resident 数达到 `page_budget` 时，只允许回收当前 feedback 提供的 `evictable_pages`
- eviction 排序现在也不再只看“和当前 target 是否同 lineage”；如果别的 active request lineages 仍然挂着，runtime 会先回收与所有 active requests 都无关的页，再优先回收较晚 request、且离对应 frontier 更远的 lineage page
- `VisibilityVirtualGeometryFeedback` 现在还会显式导出 `hot_resident_pages`，而 runtime host 会把它缓存成 `current_hot_resident_pages`；因此 current-frame feedback completion 与 next-frame prepare recycle 都会优先回收 colder unrelated page，而不会把刚刚仍在支撑 split-merge frontier 的 resident page 当普通可回收页先踢掉
- 如果本帧没有足够的可回收 budget，剩余 request 会保持 `PendingUpload`，而不是无上限扩 resident cache
- 下一帧 `build_prepare_frame(...)` 会直接观察到这次 feedback 消费后的 `Resident / PendingUpload / Missing` 变化
- 当 ancestor cascade request 仍然 pending 时，visibility planning 现在还会持续保护同一 visible frontier 下最深的 resident hidden descendants，不再在第二个 collapsed frame 就把它们重新放回 `evictable_pages`

这条实现仍然不是最终的 GPU streaming 路线：

- 还没有真实 GPU copy/upload backend
- 还没有 DMA/readback 或 page completion 信号源
- 但 `VisibilityVirtualGeometryFeedback` 已经不再是死数据结构，后续真正的 uploader/readback 只需要替换 fulfillment 来源，而不需要再重拆 runtime host 与 render-server façade 的边界

### M5 Hybrid GI Preprocess And Runtime Host Baseline

在 capability-slot 与 `GlobalIllumination` pass skeleton 之上，当前又把 `Hybrid GI` 从“统一前处理 + CPU runtime host”推进到 renderer-local GPU completion source，但仍然刻意停在 scene/probe 表示、request planning、probe cache host 与 completion readback，不伪装真正的 radiance cache shading 或 RT lighting 已经存在。

当前新增的 scene/extract 合同是：

- `RenderHybridGiExtract`
  - `probe_budget`
  - `tracing_budget`
  - `probes`
  - `trace_regions`
- `RenderHybridGiProbe`
  - `entity`
  - `probe_id`
  - `position`
  - `radius`
  - `resident`
  - `ray_budget`
- `RenderHybridGiTraceRegion`
  - `entity`
  - `region_id`
  - `bounds_center`
  - `bounds_radius`
  - `screen_coverage`

`VisibilityContext` 当前新增三组 `Hybrid GI` 前处理输出：

- `hybrid_gi_active_probes`
- `hybrid_gi_update_plan`
- `hybrid_gi_feedback`

其中计划与反馈边界被固定成：

- `VisibilityHybridGiUpdatePlan.resident_probe_ids`
- `VisibilityHybridGiUpdatePlan.requested_probe_ids`
- `VisibilityHybridGiUpdatePlan.dirty_requested_probe_ids`
- `VisibilityHybridGiUpdatePlan.evictable_probe_ids`
- `VisibilityHybridGiUpdatePlan.scheduled_trace_region_ids`
- `VisibilityHybridGiFeedback.active_probe_ids`
- `VisibilityHybridGiFeedback.requested_probe_ids`
- `VisibilityHybridGiFeedback.evictable_probe_ids`

这一层的规则当前是：

- `build_hybrid_gi_plan(...)` 只消费统一 visibility 里的 `visible_entities`
- probe 会再做一次 sphere frustum test；trace region 也会按同一视图做 bounds 过滤
- active probe 排序按 `ray_budget` 优先，再按 `probe_id` 做稳定 tie-break
- trace region 排序按 `screen_coverage` 优先，再按 `region_id` 做稳定 tie-break
- `probe_budget` 会截断本帧真正发起的 non-resident probe request
- `tracing_budget` 会截断本帧真正进入 trace schedule 的 region 数
- `VisibilityHistorySnapshot` 当前额外保留 `hybrid_gi_requested_probes`，让 `dirty_requested_probe_ids` 只记录跨帧新增 request
- resident 但本帧不再 active 的 probe 会进入 `evictable_probe_ids`，作为 viewport cache eviction 的稳定前置信号；但当某条 hierarchy request 仍然挂在当前 active frontier 上时，系统现在会继续保护这条 frontier 下最深的 resident hidden descendant probe，不再让 runtime-host cache policy 提前把它冷却回收

在 preprocess 之上，当前又补上了 viewport 级 `HybridGiRuntimeState`，固定承担：

- probe resident budget
- probe `ray_budget` metadata 记账
- resident probe -> slot 映射
- pending probe update 队列
- 本帧 `scheduled_trace_region_ids`
- 本帧 `evictable_probe_ids`

`ViewportRecord` 当前新增 `hybrid_gi_runtime`，`WgpuRenderFramework::submit_frame_extract(...)` 现在会在 `BuiltinRenderFeature::GlobalIllumination` 真正进入有效 compiled pipeline 时：

- 先用 `register_extract(...)` 同步 extract 的 probe metadata 与 resident baseline
- 再用 `ingest_plan(...)` 吞入 visibility 生成的 resident/requested/dirty/evictable probe 计划以及 trace schedule
- capability gate 关闭时彻底移除 runtime host，而不是维持一个“伪激活”缓存
- 最后把 runtime host 规模写回 façade stats

这一层的规则当前是：

- `RenderHybridGiExtract.probe_budget` 当前会同时约束 CPU fallback runtime host 的 resident probe budget，并且至少覆盖 extract 自身声明的 resident baseline probe
- extract 中声明 `resident = true` 的 probe 会获得稳定 slot，不会在重复提交时重新编号
- 只有 `dirty_requested_probe_ids` 会转成 pending update，因此重复请求不会反复入队
- `apply_evictions(...)` 释放出来的 slot 会被后续 `fulfill_updates(...)` 优先复用
- 当前 headless `wgpu` 基线依旧不会创建有效 `HybridGiRuntimeState`，因为 `hybrid_global_illumination_supported` 的 capability gate 仍然关闭

render-server façade 当前也开始把这条前处理链与 runtime host 的规模暴露到 `RenderStats`：

- `last_hybrid_gi_active_probe_count`
- `last_hybrid_gi_requested_probe_count`
- `last_hybrid_gi_dirty_probe_count`
- `last_hybrid_gi_cache_entry_count`
- `last_hybrid_gi_resident_probe_count`
- `last_hybrid_gi_pending_update_count`
- `last_hybrid_gi_scheduled_trace_region_count`

这些计数与 Virtual Geometry 一样，只有在 `GlobalIllumination` 真正进入有效 compiled pipeline 时才会写入。当前 pure `wgpu` headless baseline 已经会把这条非 RT Hybrid GI baseline 映射为 capability-supported，因此在 profile 显式 opt-in 且 extract 提供 payload 时，这些值会进入真实统计链；RT/AS 相关能力仍然保持关闭。

### M5 Hybrid GI Feedback Streaming Baseline

在 runtime-host baseline 之后，当前又把此前未消费的 `VisibilityHybridGiFeedback` 接进了 viewport probe-cache 宿主。这样 pending probe update 与 trace schedule 不再只是停留在 `VisibilityContext` 的 request/feedback 输出里，而会在帧后按 runtime budget 推进到下一帧宿主状态。

这一层当前新增的合同是：

- `HybridGiRuntimeState::consume_feedback(&VisibilityHybridGiFeedback)`
- `WgpuRenderFramework::submit_frame_extract(...)` 现在会在 render 完成后消费当前帧 feedback，再把更新后的 probe-cache runtime host 留在 viewport record
- façade stats 使用的是 feedback 消费之后的 runtime snapshot，因此当 capability 未来放开时，`cache-entry / resident-probe / pending-update / scheduled-trace` 规模会反映 submit 完成后的宿主状态

这一层的规则当前是：

- feedback 的 `scheduled_trace_region_ids` 会直接写入 runtime host
- 只有当前 feedback 中仍然处于 pending 的 `requested_probe_ids` 才会被尝试 promote
- resident 数达到 `probe_budget` 时，只允许回收当前 feedback 提供的 `evictable_probe_ids`
- 如果本帧没有足够的可回收 budget，剩余 probe update 会保持 `PendingUpdate`，而不是无上限扩 probe cache

这条实现仍然不是最终的 GI 执行路径：

- 还没有真实 traced radiance-cache kernel，当前只是把 probe/trace 场景元数据折叠进 quantized trace-region-localized GPU completion kernel
- 还没有 RT tracing backend 或更高阶 scene representation
- 但 `VisibilityHybridGiFeedback` 已经不再是死数据结构，后续真正的 tracing/update backend 只需要替换 fulfillment 来源，而不需要重拆 viewport runtime host 与 render-server façade 的边界

### M5 Hybrid GI Requested-Lineage Runtime-Source Continuation

在 `HybridGiRuntimeState` 已经能保存 requested-lineage resolve weight、irradiance history 与 RT-lighting history之后，当前又补上了一层更深的 no-schedule runtime-source continuation：`build_resolve_runtime()` 不再只在 resident-ancestor gather 成功时输出 hierarchy RT-lighting；当 pending/nonresident probe 当前拿不到 resident hierarchy gather，但 scene-driven requested-lineage support 仍然有效时，runtime host 现在也会沿 probe 自身与 `parent_probe_id` chain 重新编码 runtime RT history。

这一层新增的规则是：

- `runtime_hierarchy_rt_lighting(...)` 在 resident ancestor gather 为零时，不再直接返回 `None`；它会转入 `direct_lineage_rt_lighting_fallback(...)`
- 这条 fallback 只在 probe 当前不在 `resident_slots` 时启用，避免 resident gather 与 direct runtime source 互相污染
- fallback 会先尝试当前 probe 自己已经持有的 `probe_rt_lighting_rgb`
- 如果 probe 自己没有足够 runtime RT history，则会继续沿 `parent_probe_id` chain 收集带历史 RT-lighting 的 ancestor
- probe 自身与 ancestor history 都会继续乘上 requested-lineage support、ray budget 与 runtime RT intensity，因此 runtime source 不再只是“有颜色就回灌”的裸回退
- 因为 `runtime_trace_source(...)` 本来就优先消费 `hierarchy_rt_lighting(probe_id)`，所以这条更深的 requested-lineage RT continuation 会直接进入 pending probe GPU prepare/readback，而不需要再额外开一条只给 encode/post-process 用的旁路
- 对没有 `parent_probe_id` 的 standalone pending probe，runtime host 现在还会额外保留一条 direct-RT fallback：即使 hierarchy resolve weight 仍是 flat baseline，只要 probe 自己已经拥有 GPU-produced `probe_rt_lighting_rgb`，`build_resolve_runtime()` 也会把它重新编码为 lightweight hierarchy continuation，让 `runtime -> GPU prepare` 不再因为“无 lineage 但有 direct RT history”而直接掉回黑值

这意味着 `Hybrid GI` 现在已经不只是在 irradiance 支路上保住 requested-lineage runtime source；RT hybrid-lighting 也开始沿同一份 scene-driven lineage truth 继续压回 `runtime -> GPU prepare` 主链。后续若要继续推进 screen-probe hierarchy gather / request / radiance-cache update / RT hybrid lighting，就不需要再重拆 runtime host 与 GPU source 的边界。

### M5 Virtual Geometry Explicit Draw-Ref Authority And Cluster-Raster Submission Continuation

本轮又把 `Virtual Geometry` 的 unified-indirect authority 往下压了一层，不再让 shader 和 renderer 分别从 buffer 顺序与 `pending_draw.indirect_draw_ref` 二次重建 submission truth。

当前新的收口点是：

- `build_shared_indirect_args_layout(...)` 现在会同时产出：
  - authoritative `draw_ref_buffer` record
  - per-draw `submission_token`
  - per-draw `VirtualGeometrySubmissionDetail`
- `VirtualGeometryIndirectDrawRefInput` 现在显式携带：
  - `segment_draw_ref_count`
  - `submission_token`
- `virtual_geometry_indirect_args.wgsl` 不再通过扫描 `draw_ref_buffer` 重算 draw-ref rank；cluster-raster compaction 与 `first_instance` / debug submission token 都开始直接消费 shared layout 写下来的显式 authority
- `build_mesh_draws(...)` 现在也优先从 shared layout 回填 `VirtualGeometrySubmissionDetail`，而不是把 renderer 侧 draw-level submission detail 再从 `pending_draw.indirect_draw_ref` 和当帧排序残留拼回去
- 当前吸收后的 `zircon_runtime/src/graphics/**` 路径里，这份 authority 又继续往下压了一层：`VirtualGeometrySubmissionDetail` 与 shared layout / last-state submission records 现在显式携带 `draw_ref_index`，`virtual_geometry_indirect_stats(...)` 与 `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 也优先消费这份 explicit draw-ref truth，而不是继续把 `indirect_args_offset / stride` 当成 execution-side draw-ref identity 的默认来源
- 同一条 compute pass 现在还会写出 GPU-generated `SubmissionAuthorityRecord(draw_ref_index, entity, page_id, submission_token)` sidecar，并沿 `BuiltMeshDraws -> VirtualGeometryIndirectStats -> SceneRenderer` last-state 挂住；因此 deepest submission fallback 在 CPU records、execution records、indirect args、submission token、draw-ref/segment buffer 全部缺失时，仍然可以通过 actual execution indices + GPU authority sidecar 恢复真实 submission records
- 这份 GPU authority sidecar 现在又继续扩成完整 execution template：`draw_ref_index + cluster span/count + page/slot/state + lineage/lod/frontier + submission_index/draw_ref_rank + entity`。因此 `read_last_virtual_geometry_indirect_execution_records()` 与 `read_last_virtual_geometry_indirect_execution_segments_with_entities()` 在 host-built `execution_records_buffer` 缺失时，也已经能直接通过 `execution indices + GPU authority template` 恢复 actual execution subset，而不再只剩 submission-record 这一层有 deeper GPU fallback

这让同一份 shared layout 同时成为：

- renderer draw submission detail 的真值
- GPU draw-ref compaction 的真值
- cluster-raster indirect args `first_instance` 的真值
- execution stats / last-state submission readback 的显式 draw-ref 真值
- deeper submission readback fallback 的 GPU-generated authority 真值
- deeper execution-record / execution-segment readback 的 GPU-generated authority 真值

因此 M5 Virtual Geometry 当前已经从“CPU 先排序、GPU 再跟随”继续推进到“CPU/GPU 都消费同一份 visibility-owned indirect authority”。下一条自然延伸就是把这份 authority 再继续压进更真实的 GPU-generated args compaction source / indirect execution ownership，而不再只停在 current shared-buffer contract。

### M5 Hybrid GI Scene-Driven Parent-Chain Runtime Gather

`Hybrid GI` 这边，本轮继续把 hierarchy-aware resolve/runtime source 从 host 预编码的 exact probe map 推进到 encode-side scene gather。当前 `runtime_trace_source(...)` 与 `runtime_irradiance_source(...)` 已经不再只接受 “当前 probe 在 `HybridGiResolveRuntime` 里有没有 exact entry” 这一个条件。

新增的 scene-driven gather 规则是：

- 如果当前 probe 没有 exact runtime hierarchy irradiance / RT source，encode side 会沿当前 extract 的 `parent_probe_id` chain 继续向上找 ancestor runtime source
- gather 结果会按 parent-chain depth 做衰减，再重新量化成 GPU prepare consumption 用的 `support_q + packed_rgb`
- `pending_probe_inputs(...)` 与 `resident_probe_inputs(...)` 都统一改走这条 parent-chain runtime gather，而不是仅在 host 里预先把每个 child probe 的 exact source 编好

这一步的重要性在于：screen-probe hierarchy continuity 现在已经真正进入“scene-driven runtime gather”阶段。也就是说，当前 frame 的 probe hierarchy 拓扑本身开始决定 runtime irradiance / RT history 怎样继续流进 GPU prepare，而不再要求 `build_resolve_runtime()` 预先为每个 descendant probe 填满一张 exact source 表。后续继续推进 deeper screen-probe hierarchy / probe gather / RT hybrid lighting 时，可以直接在这条 parent-chain gather contract 上扩 scene representation、probe request、radiance-cache resolve，而不需要再回头拆 encode/runtime 的边界。

## M3 Visibility Preprocess Baseline

这一轮已经从 “只有 batch/instancing 结构” 进入到真正消费视图数据的 M3 基线，但仍然刻意停在统一前处理层，不把后续 draw submission 或 feature 私有逻辑提前耦合进去。

当前已经固定下来的前处理边界是：

- `zircon_framework::render::RenderMeshSnapshot` 现在显式携带 `mobility` 和 `render_layer_mask`，让 legacy snapshot 兼容桥也能保留 visibility 元数据
- `zircon_framework::render::VisibilityInput` 新增 `renderables`，元素类型为 `VisibilityRenderableInput`
- `World::build_viewport_render_packet(...)` 会按 `node_id` 稳定排序 mesh/light extract，避免 `HashMap` 迭代顺序把 batch key 和缓存行为变成随机结果
- `SceneViewportExtractRequest` 现在只携带中性的 `ViewportRenderSettings + active_camera_override + camera + viewport_size`；selection 和 editor tool/grid 状态已经不再进入 runtime world extract 路径
- `ViewportRenderSettings` 把 runtime world 真正需要的 `projection_mode / display_mode / preview_lighting / preview_skybox` 从 `SceneViewportSettings` 里拆了出来，避免 `zircon_scene` 继续依赖整包 editor authoring 状态
- `ViewportCameraSnapshot` 新增 `aspect_ratio`，`RenderFrameExtract` 提供 `apply_viewport_size(...)` / `with_viewport_size(...)`，让 scene extract 自己持有真实视口纵横比
- editor/runtime submit bridge 会在提交前补丁 `RenderFrameExtract` 的 viewport size，作为现阶段 consumer 侧安全网；真正的相机/视图语义仍然归 `zircon_scene`
- camera gizmo frustum overlay 现在直接使用 extract 上的 `aspect_ratio`，不再退回硬编码 `16:9`
- 2026-04-19 的 contract cutover 之后，`SceneViewportSettings`、`ViewportRenderSettings`、`SceneViewportExtractRequest`、`SceneViewportTool`、`TransformSpace`、`ViewOrientation`、`GridMode` 已固定由 `zircon_framework::render` 直接提供；`zircon_scene` 保留 world/render authority，但不再从根 crate 转发这组 editor authoring/request 类型
- 同日继续推进到 `zircon_graphics` 内部后，`DisplayMode`、`ProjectionMode`、`ViewportCameraSnapshot`、`RenderFrameExtract`、`RenderSceneSnapshot`、overlay/icon DTO、Hybrid GI extract、Virtual Geometry extract 也都已经直接从 `zircon_framework::render` 导入；graphics 继续经 `zircon_scene` 保留的只剩 `World`、`Mobility`、`EntityId`、`default_render_layer_mask` 这类 runtime authority
- 继续收尾到 `zircon_scene` 自身之后，root `lib.rs` 里剩余的 framework-owned render re-export 也已删除；`zircon_scene/tests/render_frame_extract.rs` 与 `zircon_scene/tests/viewport_packet.rs` 现在同样直接从 `zircon_framework::render` 导入这些 DTO，不再经 `zircon_scene` 根级入口旁路 owner
- `zircon_graphics::VisibilityContext` 不再只是三组 entity id，而是会基于 mesh extract 生成稳定的 `VisibilityBatchKey` / `VisibilityBatch`
- `VisibilityContext` 现在额外暴露 `visible_entities`、`culled_entities`、`visible_batches`，把 “结构化 batch” 与 “视图裁剪结果” 明确分层
- frustum culling 当前已经进入统一 visibility 前处理：同时支持 perspective / orthographic camera，并使用 view-space sphere test 过滤 mesh batch 成员
- 当前 culling 半径仍然是保守占位实现：`radius = mesh.transform.scale.abs().length() * 0.5`。这只是给后续真实 mesh bounds、BVH、cluster/virtual geometry 接入预留槽位，不是最终几何界限模型
- `gpu_instancing_candidates` 现在只来自 `visible_batches`，因此同一个 structural batch 只要有成员被裁掉，就不会继续把整组实例错误地当成当前 pass 可直接实例化的可见 draw
- `VisibilityContext` 现在还会生成连续的 `visible_instances` 与 `draw_commands`：前者是按稳定 batch 顺序压实后的实例列表，后者只保存 `visible_instance_offset + visible_instance_count + batch key`，作为后续 indirect draw/upload buffer 的中立脚手架，而不是直接绑定某个后端的原生参数结构
- `VisibilityContext::from_extract_with_history(...)` 现在支持把统一前处理扩到 BVH/AS 脏区规划：`bvh_instances` 提供当前场景实例边界，`history_snapshot` 记录上一帧可比较状态，`bvh_update_plan` 输出 `FullRebuild / Incremental` 策略以及 inserted/updated/removed entity 集合
- `VisibilityContext` 现在还会输出 `instance_upload_plan`：把当前实例按 static/dynamic 拆开，并且只把需要重传的 dynamic entity 放进 `dirty_dynamic_entities`，从而把实例上传准备也收回统一前处理层，而不是交给各个 mesh/RT feature 自己重新 diff
- `VisibilityContext` 现在也会消费 `ParticleExtract.emitters`，并输出 `particle_upload_plan`：当前阶段只做 emitter membership 级别的 `dirty_emitters / removed_emitters` 规划，不伪造完整粒子模拟参数，但已经把粒子上传准备接进同一条历史 diff/上传准备边界

这条实现故意保持“只做前处理结构，不做后续 draw submission 特化”的边界：

- 目前没有把 occlusion culling、真实 BVH 节点构建、TLAS/BLAS 上传、RT instance buffer 编码直接塞进 `VisibilityContext`
- 目前也没有让具体 feature 在 pass 里各自重新做一遍分类；统一 batch 结果仍然归 `visibility/` 负责，后续只是在这里继续向下加层
- `RenderPipelineAsset::compile(...)` 仍然保持 asset/feature graph 编译职责，不把 per-frame visibility/batching 混进 pipeline asset 编译期

## Validation Status

当前已验证通过的内容：

- `zircon_rhi` 描述符与 capability 基线
- `zircon_rhi_wgpu` 的 capability fallback
- `zircon_render_graph` 的排序与 cycle rejection
- `zircon_framework` 的稳定 handle/type 契约
- `RenderFramework::query_stats()` 的 capability plumbing：`wgpu` 基线能力现在会通过 `RenderCapabilitySummary` 进入 façade 侧统计快照，供上层按 capability 做 feature gate，而不需要直接接触 `zircon_rhi` / `wgpu`
- `RenderQualityProfile` 的 pipeline override：viewport 在没有显式 pipeline 绑定时，quality profile 现在可以稳定选择 built-in deferred pipeline 作为默认 renderer
- `RenderQualityProfile` 的 M4 feature toggles：当前可以直接控制 `clustered lighting / SSAO / history resolve` 以及 async-compute 偏好，而不需要 consumer 直接接触 renderer 内部类型
- `RenderQualityProfile` 的 M5 flagship toggles：当前可以对 `virtual geometry / hybrid global illumination` 发出 opt-in 请求，同时继续通过 capability gate 保证纯 `wgpu` 基线不会被 profile 强行打开
- `RenderStats.last_frame_history`：render façade 现在会把最新 viewport history handle 暴露到统计快照，便于验证跨帧资源宿主是否稳定工作
- `RenderStats.last_effective_features / last_async_compute_pass_count`：render façade 现在能暴露当前 pipeline 在 quality/capability 处理后的真正 feature 集和 async-compute 退化结果
- `RenderStats.capabilities.virtual_geometry_supported / hybrid_global_illumination_supported`：render façade 现在会把旗舰功能是否具备 backend 支撑显式暴露给上层
- `RenderStats.last_virtual_geometry_visible_cluster_count / last_virtual_geometry_requested_page_count / last_virtual_geometry_dirty_page_count`：render façade 现在会暴露 Virtual Geometry 前处理规模；当 feature 未启用或 extract 为空时它们保持 `0`
- `RenderStats.last_virtual_geometry_page_table_entry_count / last_virtual_geometry_resident_page_count / last_virtual_geometry_pending_request_count / last_virtual_geometry_completed_page_count / last_virtual_geometry_replaced_page_count`：render façade 现在会暴露 Virtual Geometry runtime host 与 GPU uploader completion 的 page-table / resident / pending / completed / replaced 规模
- `RenderStats.last_virtual_geometry_indirect_draw_count / last_virtual_geometry_indirect_args_count / last_virtual_geometry_indirect_segment_count`：render façade 现在会同时暴露 prepare-owned unified indirect draw / args / segment totals，让 shared indirect contract 的 cardinality 对上层可见
- `RenderStats.last_virtual_geometry_execution_segment_count / last_virtual_geometry_execution_page_count / last_virtual_geometry_execution_resident_segment_count / last_virtual_geometry_execution_pending_segment_count / last_virtual_geometry_execution_missing_segment_count / last_virtual_geometry_execution_repeated_draw_count`：render façade 现在还会暴露真实 Virtual Geometry execution subset 的 unique segment/page/state/compaction summary，因此上层不需要读 renderer 私有 GPU readback，也能区分“prepare 里有多少东西”与“本帧真正执行了多少东西”
- `RenderStats.last_hybrid_gi_active_probe_count / last_hybrid_gi_requested_probe_count / last_hybrid_gi_dirty_probe_count`：render façade 现在已经预埋 Hybrid GI 前处理计数，但当前 `wgpu` capability gate 仍会把它们保持在 `0`
- `RenderStats.last_hybrid_gi_cache_entry_count / last_hybrid_gi_resident_probe_count / last_hybrid_gi_pending_update_count / last_hybrid_gi_scheduled_trace_region_count`：render façade 现在还会暴露 Hybrid GI runtime host 的 probe cache / resident probe / pending update / trace schedule 规模；当前 `wgpu` capability gate 关闭时它们同样保持 `0`
- `zircon_scene` 的 `RenderFrameExtract <-> RenderSceneSnapshot` 适配
- `zircon_graphics::runtime::WgpuRenderFramework` 的 viewport 创建、pipeline/profile 设置、frame submit 与 stats 更新
- `zircon_graphics::pipeline::RenderPipelineAsset::compile(...)` 的确定性编译、duplicate stage/feature rejection，以及 `DebugOverlay` 独立 extract 依赖
- `zircon_graphics::pipeline::RenderPipelineAsset::default_deferred()` 的第二条内建 pipeline：固定 deferred stage/pass 顺序、built-in handle lookup，以及 `RenderFramework` 侧的 built-in deferred pipeline 选择
- `zircon_graphics::pipeline::RenderPipelineAsset::compile(...)` 的 M4 compile contract：当前会稳定聚合 `history_bindings`，并把 built-in Forward+ / Deferred 编译到 `ssao-evaluate -> clustered-light-culling -> history-resolve` 这一组新的 pass 链
- `zircon_graphics::pipeline::RenderPipelineAsset::compile_with_options(...)`：当前已经支持显式禁用 M4 feature 和 async-compute lane fallback，从而让 quality profile / capability 能真正参与 built-in pipeline 编译
- `zircon_graphics::pipeline::RenderPipelineAsset::compile_with_options(...)` 的 M5 opt-in contract：当前已经支持用 `with_feature_enabled(...)` 显式打开 `VirtualGeometry` 与 `GlobalIllumination`，同时保证默认 Forward+ / Deferred 编译结果不被旗舰槽位污染
- `zircon_scene` 的 visibility 元数据保留：`RenderFrameExtract` 现在会保留 static/dynamic 分区、render layer mask，以及稳定排序后的 mesh/light extract
- `zircon_scene` 的 camera aspect propagation：viewport size 会进入 `SceneViewportExtractRequest`、`ViewportCameraSnapshot`、`RenderFrameExtract`，并同步到 camera gizmo frustum overlay
- `zircon_graphics/src/tests/project_render.rs` 当前也已经直接从 `zircon_framework::render` 导入 viewport request / scene packet DTO，避免再通过 `zircon_scene` 根级入口旁路 render contract ownership
- `zircon_graphics` 当前已经把生产代码里剩余的 `RenderFrameExtract` / `RenderSceneSnapshot` / `DisplayMode` / `ProjectionMode` / `ViewportCameraSnapshot` / overlay DTO / Hybrid GI / Virtual Geometry extract 与 scene semantics 消费面全部切到 `zircon_framework::{render,scene}`；`zircon_scene` 在 graphics crate 内只剩 dev-only tests fixture 依赖，用来构造 `World` 级 runtime-authority 回归
- 同一轮还顺手删掉了 `zircon_graphics` root 上无人消费的 `LegacyRenderService` / `LegacyRuntimePreviewRenderer` / `LegacySharedTextureRenderService` compat export，避免 graphics root 继续保留兼容噪音
- `zircon_scene/src/lib.rs` 当前也已经彻底收掉 root render contract re-export；repo 内剩余从 `zircon_scene` 导入的 surface 仅保留 `World`、`Mobility`、`EntityId`、`NodeKind`、`SceneProjectError` 等 runtime authority / scene domain 类型
- `zircon_scene` 的 M5 extract 预埋：legacy snapshot adapter 当前会稳定初始化 `virtual_geometry = None` 与 `hybrid_global_illumination = None`
- `zircon_framework::render` 的 Virtual Geometry preprocess contract：当前已经公开 `RenderVirtualGeometryCluster`、`RenderVirtualGeometryPage` 与扩展后的 `RenderVirtualGeometryExtract`
- `zircon_framework::render` 的 Hybrid GI preprocess contract：当前公开面已经收口为 settings/budget/debug 型 `RenderHybridGiExtract`，并把 `RenderSceneGeometryExtract` 的灯光输入扩成 `directional_lights / point_lights / spot_lights`；旧 `RenderHybridGiProbe / RenderHybridGiTraceRegion` 只保留为 crate-internal fixture bridge，不再代表长期 public authoring API
- `zircon_graphics::VisibilityContext` 的 M3 基线：稳定 batch key、deterministic batch ordering、frustum culling、visible/culled 分区，以及只对真正重复且仍然可见的动态 batch 暴露 instancing candidates
- `zircon_graphics::VisibilityContext` 的 GPU-driven 脚手架：`visible_instances` / `draw_commands` 会按稳定 batch 顺序压实可见实例，为后续 indirect draw args 与 instance upload 提供一致入口
- `zircon_graphics::VisibilityContext` 的 BVH dirty/update 框架：无历史时回退 `FullRebuild`，有历史时输出 `Incremental` 的 inserted/updated/removed entity 集合，且继续复用统一的保守 sphere bounds 占位模型
- `zircon_graphics::VisibilityContext` 的实例上传准备：`instance_upload_plan` 会分离 static/dynamic 实例，并且只标记本帧需要重传的 dynamic entity，避免把后续 instance upload policy 分散回 renderer feature
- `zircon_graphics::VisibilityContext` 的粒子上传准备：`particle_upload_plan` 会在没有历史时对全部 emitter 做全量上传，在有历史时只标记新增/移除的 emitter，为未来真正的粒子 GPU buffer/upload policy 预埋统一入口
- `zircon_graphics::VisibilityContext` 的 Virtual Geometry 前处理：当前已经能输出 cluster-level 可见集、resident/requested/dirty/evictable page 计划、稳定的 feedback 请求集合，以及按 parent lineage 保留边界的 `VisibilityVirtualGeometryDrawSegment`
- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 的 prepare snapshot：当前已经能把 resident/pending/evictable page 与 visible cluster 合成为 `VirtualGeometryPrepareFrame`，并直接消费 visibility-owned `draw_segments` 生成显式 `cluster_draw_segments + available_slots`
- `zircon_graphics::VisibilityContext` 的 Hybrid GI 前处理：当前已经能输出 active probe、resident/requested/dirty/evictable probe 计划，以及稳定的 trace schedule / feedback 请求集合
- `zircon_graphics::runtime::WgpuRenderFramework` 的 viewport history host：当前已经能在兼容的重复提交间复用 `FrameHistoryHandle`，并在 pipeline 切换时轮换 handle，同时继续复用统一 visibility history 作为跨帧 diff 输入
- `zircon_graphics::runtime::WgpuRenderFramework` 的 M4 quality/capability mapping：当前会把 profile/caps 编译成有效 pipeline，headless `wgpu` 会把 async-compute pass cleanly 降级到 graphics queue，并把 effective feature 结果写回 façade stats
- `zircon_graphics::feature::BuiltinRenderFeature` 的 M5 skeleton：当前 opt-in 后会稳定编译出 `virtual-geometry-prepare` 与 `hybrid-gi-resolve`，并把 `GlobalIllumination` history slot 聚合进 `history_bindings`
- `zircon_graphics::runtime::WgpuRenderFramework` 的 M5 capability map：headless `wgpu` 当前会把 `virtual_geometry_supported / hybrid_global_illumination_supported` 暴露为 `true`，用于表示当前 graphics/offscreen baseline 已可运行；`acceleration_structures_supported / inline_ray_query / ray_tracing_pipeline` 仍保持 `false`，从而把 RT-only 路径继续关在更高 capability tier 里
- `zircon_graphics::runtime::WgpuRenderFramework` 的 Virtual Geometry stats plumbing：当前 submit 路径已经能把 Virtual Geometry 可见 cluster / requested page / dirty page 数，以及 runtime host 的 page-table / resident / pending-request / completed / replaced 规模写回 façade stats；在 capability/profile/extract 都满足时，这些值会进入真实统计链
- `zircon_graphics::runtime::WgpuRenderFramework` 的 Virtual Geometry indirect-raster stats：当前 façade 还会暴露 `last_virtual_geometry_indirect_draw_count` 与 `last_virtual_geometry_indirect_buffer_count`，用于证明 VG prepare segment 不只是被编译打开，而且真的以 compute-generated shared indirect args buffer + per-draw offset 的方式提交了 renderer-local indirect raster draw
- `zircon_graphics::scene::SceneRenderer` 的 frame orchestration 边界：`scene_renderer_render/`、`scene_renderer_render_with_pipeline/` 与共享 `scene_renderer_runtime_outputs/` 当前都已经拆成子树，snapshot render、compiled-pipeline render、last readback/indirect stats reset 与 store 不再回流成根级聚合脚本
- `zircon_graphics::runtime::WgpuRenderFramework` 的 submit/runtime host 边界：`submit_frame_extract/` 里的 context、runtime-prepare、record、stats 以及 `Hybrid GI / Virtual Geometry` 的 prepare/completion helper 当前都已经下沉到 folder-backed 子树，根入口只保留 orchestration wiring，避免 render-server runtime host 继续回流成新聚合文件
- `zircon_graphics::visibility::build_virtual_geometry_plan(...)` 的 hierarchy refine baseline：当前已经支持 `parent_cluster_id` 驱动的 budget-aware refine frontier，并把 request 侧的 `streaming_target_clusters` 与 resident-gated `visible_clusters` 明确分开；children/grandchildren page 还没 resident 时，coarse frontier 会继续留在当前帧 raster，而 request 仍然会继续追更细 hierarchy
- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 的 feedback consumption baseline：当前 pending page request 会在 resident budget 内消费 feedback 并推进为 resident；没有可回收 budget 时则会继续保持 pending
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry GPU completion baseline：当前 renderer 已经会把 resident page table、带 `size_bytes` 的 pending request，以及 prepare snapshot 提供的 `available_slots / evictable_slots` 上传到真实 `wgpu` storage buffer，按 renderer-local `streaming_budget_bytes + reclaimable_bytes` 做 size-aware uploader arbitration，并把 post-uploader `page_table_entries / completed_page_assignments(page_id, slot)` 通过 readback 返回给 runtime host
- `zircon_graphics::runtime::WgpuRenderFramework` 的 Virtual Geometry post-render progression：当前 submit 路径已经会优先消费 renderer GPU readback 的 `page_table_entries + completed_page_assignments`，再回退到 `VisibilityVirtualGeometryFeedback`，从而让下一帧 prepare snapshot 可以观察到 GPU-truth residency 变化，并且 host 不再重排 GPU 选定的 page-slot ownership；在 render frontier 侧，刚完成上传的 children page 还会被一帧 split hysteresis 暂时保护，避免 coarse parent 在 upload 完成帧立刻消失；与此同时 runtime host 在接收新 extract 时也会主动裁掉已经离开场景的 stale page，不再把旧 page-table / pending-request truth 带进下一帧
- `zircon_graphics::visibility::build_virtual_geometry_plan(...)` 的 wider split-merge policy：当前除了 upload-completion split hold 与 split 落地帧 coarse-parent hold 之外，还会在 frontier 从 resident children 回退到 parent 的当帧继续保护仍 resident 的 child page，一帧之后才重新允许它进入 `evictable_pages`；最新一层还会把这些 hidden-but-still-hot resident frontier page 显式导出为 `VisibilityVirtualGeometryFeedback.hot_resident_pages`，继续下沉到 runtime residency host 的 recycle order
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry fallback consumption：当前当 `VirtualGeometry` feature 显式开启且 frame 挂有 prepare snapshot 时，mesh fallback path 不仅会 honor `visible_entities` 过滤结果，还会直接消费 `cluster_draw_segments` 提供的 entity-cluster index slice，并把 `prepare.visible_clusters[*].resident_slot` 带进 slot-aware tint/brightness 规划，再通过 `MeshDraw.first_index + draw_index_count + tint/brightness` 在 base/prepass/deferred 三条 raster 路径上消费这些 cluster fallback；即使 prepare 显式覆盖 segment ordinal，最终离屏输出也会跟着 prepare 走
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry indirect raster baseline：当前 prepare 驱动的 fallback draw 会先把 visibility-owned `cluster_start_ordinal / cluster_span_count / cluster_total_count / state` 编成 GPU input，再由专用 compute pipeline 生成 shared indirect args，并在 base/prepass/deferred 三条 raster 路径上改走 `draw_indexed_indirect(...)`；这些 args 已经不再按 draw 单独分配 `wgpu::Buffer`，而是聚合成 frame-shared indirect args buffer 再用 per-draw offset 消费，而 unified indirect ownership 现在先由 visibility 侧的 lineage-aware `draw_segments` 决定，再以 prepare 的 `cluster_draw_segments` 为真值继续下沉到 renderer，renderer 不会再对显式 prepare segment 做二次 regroup；最新一层 last-state 现在还会额外保留并回读真实 GPU-submitted draw-ref buffer，因此测试不仅能验证 segment truth，也能验证每条提交 draw 最终引用的 segment 映射；再往下一层，`prepare.unified_indirect_draws()` 现在会先在 prepare 层按 `submission_slot / frontier_rank / page / cluster lineage` 排出第一份 authoritative order，并把这条顺序继续编码成 cluster-raster draw 的 internal `submission_index`，因此 renderer 末端不再负责发明第一份排序，只负责消费和 compaction；在这条排序之下，shared indirect segment buffer 现在也不再只从 `pending_draws` 反推 unique segment 列表，而是会先吃 prepare/visibility authoritative segment truth，再并上 pending-only fallback segment，因此即使某些实体因为当前 mesh filtering 没有生成 pending draw，真实 GPU-submitted segment buffer 仍会保留 prepare-owned visibility authority；最新一层 shared args build 还会先从 scene mesh + prepare-owned cluster draws 生成 authoritative draw-ref records，再让 `pending_draws` 只负责补 fallback key 与回填真实 draw offsets，所以 `draw_ref_buffer / indirect args` source 也开始脱离 CPU pending-draw existence truth，而允许“真实提交 draw 仍然是 drawable subset，但 shared args source 已经保留更宽 prepare-owned visibility truth”的状态；在此基础上，shared indirect args build 现在也会为每条 pending draw 回填按 authoritative submission order 排好的真实 args offset，而 `build_mesh_draws(...)` 还会继续按这条 authoritative offset 稳定重排最终 `MeshDraw` 列表，因此 `draw_indexed_indirect(...)` 的真实执行顺序终于也不再绑定 CPU pending-draw 插入顺序；同 `mesh_index_count + segment_key` 的重复 primitive draw 现在还会继续折叠成共享的 indirect args / draw-ref record，使 visibility-owned unified indirect authority 不只控制排序，也开始控制真实 args cardinality
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry fallback slot submission authority：当前 unique indirect segment buffer 已经不再固定跟着 first-seen draw order，而会按 prepare 投影出来的 `submission_slot` 稳定排序；因此 draw-ref mapping、真实 GPU submission segment 顺序与 pending cluster-raster consumption 都会继续跟随 fallback recycle-slot authority 改变；最新一层 `draw_ref_buffer` 本身也已经按同一套 `submission_slot / frontier_rank / page / cluster lineage` key 排序，不再只是“固定 CPU draw 顺序上的 segment remap”
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry slot-aware cluster-raster consumption：当前 `resident_slot` 不再只影响 tint/brightness；它已经进入 GPU-generated indirect args，会改变 resident fallback 的 `first_index / index_count`，从而让不同 slot ownership 真正消费不同的 cluster-raster 子范围
- `zircon_graphics::types::VirtualGeometryPrepareFrame` 的 prepare-owned unified indirect ownership：当前 `unified_indirect_draws()` 已经退化成 prepare snapshot 投影层，只在旧 helper 没有显式写出 `page_id / resident_slot` 时从 `visible_clusters` 回填 ownership；真正的 compaction authority 固定留在 `prepare_visible_clusters(...)` 里，因此不同 resident page 不会被错误并入同一条 indirect draw，而显式 prepare segment 也不会再在 renderer 路径里被重新合并
- `zircon_graphics::runtime::WgpuRenderFramework` 的 Hybrid GI stats plumbing：当前 submit 路径已经能把 Hybrid GI active/requested/dirty probe 数，以及 runtime host 的 cache-entry / resident-probe / pending-update / scheduled-trace 规模写回 façade stats；同一条统计链现在还继续暴露了 Lumen-style Milestone 1 的 scene-driven readback 面，包括 `scene_card_count`、surface-cache resident/dirty/feedback/capture-request/invalidation 计数，以及 voxel resident/dirty/invalidation 计数；因此 façade 已经能直接观察 scene representation 是否在 editor/runtime 提交路径上稳定注册、失效和重建，而不必通过 test-only 内部状态访问器才知道 cards/pages/clipmaps 有没有变化。
- `zircon_graphics::runtime::HybridGiRuntimeState` 的 feedback consumption baseline：当前 pending probe update 会在 resident budget 内消费 feedback 并推进为 resident；没有可回收 budget 时则会继续保持 pending，同时 trace schedule 会被写回 runtime host
- `zircon_graphics::runtime::HybridGiRuntimeState` 的 renderer prepare snapshot：当前 runtime host 已经能导出 `HybridGiPrepareFrame`，把 resident probe cache、pending update、trace schedule 与 evictable probe 列表显式交给 renderer
- `zircon_graphics::scene::SceneRenderer` 的 Hybrid GI GPU completion baseline：当前 renderer 已经会把 resident probe cache、pending update、scheduled trace region ids 上传到真实 `wgpu` compute/readback 路径，并结合 `RenderHybridGiExtract` 的 probe/trace 场景元数据返回 `completed_probe_ids / completed_trace_region_ids / probe_irradiance_rgb`；多个 trace region 同时命中同一 probe 时，radiance source 现在会做归一化权重 blend，而不是简单累加到饱和；同一帧 `directional_lights` 也会先被聚合成 light seed，再把 traced radiance tint 和 energy 一起推向真实 scene lighting 的颜色/强度方向；与此同时 `parent_probe_id` 现在已经继续下沉到 resident/pending probe GPU 输入，direct parent/child 关系会真实改变 radiance-cache gather，而不再只影响 visibility frontier；再往下一层，pending probe 现在还会显式携带三级 resident ancestor id/depth，在 nonresident hierarchy gap 上继续获得 hierarchy-aware gather boost，并把更远 resident ancestor 的 radiance / traced tint 作为显式 lineage continuation 混进 pending probe update，而不是在第二层 resident ancestor 再次截断；最新一层还补上了 primary resident ancestor 的 lineage-only radiance continuation，因此 pending probe 在只有 hierarchy lineage、没有直接 spatial overlap时也不会再退化回局部 neutral trace
- `Hybrid GI` 的 Milestone-1 scene-prepare renderer seam：`HybridGiRuntimeState::build_scene_prepare_frame()` 现在会把 scene-driven `card_capture_requests + voxel_clipmaps` 挂进 `ViewportRenderFrame.hybrid_gi_scene_prepare`，`SceneRendererCore::execute_runtime_prepare_passes(...)` 再把它连同 `ResourceStreamer` 一起送进 `HybridGiGpuResources::execute_prepare(...)`；renderer 没有把 cards 和 voxels 各拆成一条新 storage-buffer，而是显式收束成 binding `4` 的 unified `scene_prepare_descriptor_buffer`，把 completed/irradiance/trace outputs 顺延到 bindings `5..8`，以避免撞上当前 `wgpu` compute-stage 仅 `8` 条 storage-buffer binding 的限制。`update_completion.wgsl` 现在也开始真实消费这份 scene descriptor buffer：附近 card-capture request 或 voxel clipmap 会正向 boost nearby probe 的 traced radiance/readback，说明 Milestone 1 已经把 `scene representation -> runtime scene prepare -> renderer buffer -> shader consumption` 主链真正打通，而不再只是把 descriptor 绑到管线里闲置。最新一层又继续补上了 per-frame card-capture atlas / capture RT scaffold：`collect_inputs(...)` 现在会把 `meshes + directional/point/spot lights` 一并带进 prepare 输入，`create_buffers(...)` 则只保留资源编排，把 card-capture 着色拆进独立 `card_capture_shading.rs`，并把 voxel residency debug 拆进独立 `voxel_clipmap_debug.rs`。这条新 seam 会优先消费 `ResourceStreamer` 已准备的 `MaterialRuntime`，必要时再回退到 `ProjectAssetManager::load_material_asset(...)`，从而把 `mesh tint * material base_color + material emissive + split direct light` 编成最小 scene-driven texel 内容写进 atlas tile / capture layer；只有解析不到 matching mesh 时才回退到 deterministic debug texel。与此同时，同一份 snapshot 现在也会为每个 resident voxel clipmap 派生一条最小 `voxel_clipmap_rgba_samples` 调试样本，并额外压出固定 `4x4x4` grid 的 `voxel_clipmap_occupancy_masks`、`voxel_clipmap_cell_rgba_samples`、`voxel_clipmap_cell_occupancy_counts` 与 `voxel_clipmap_cell_dominant_node_ids`，让 renderer seam 不只观察 voxel radiance seed 与 scene-driven spatial occupancy，还能观察最粗一层 cell-level volume content、同一 coarse voxel cell 上的重叠 contributor density，以及当前哪一个 mesh 在该 cell 上持有 dominant contributor authority，而不再只能通过 near-field probe bias 间接判断 voxel scene 是否活起来。最终这批资源再通过 `HybridGiGpuPendingReadback -> HybridGiGpuReadback` 暴露 `HybridGiScenePrepareResourcesSnapshot`，让 renderer seam 现在不只验证 descriptor consumption，还能验证 atlas/capture resource 的 slot/extent/layer truth、真实 scene-driven texel sample，以及 voxel clipmap sample/occupancy/cell-volume/cell-density/dominant-authority truth。
- 同一条 Hybrid GI scene-prepare seam 当前又继续把 voxel residency authority 往 runtime host 收：`HybridGiScenePrepareFrame` 现在除了 `card_capture_requests / voxel_clipmaps` 之外，还会带固定 `4x4x4` grid 的 `voxel_cells(clipmap_id, cell_index, occupancy_count)` payload。`collect_inputs(...)` 会把它透传到 renderer prepare，`create_buffers(...)` 在 payload 存在时会直接从 runtime-owned cell counts 生成 `voxel_clipmap_occupancy_masks` 与 `voxel_clipmap_cell_occupancy_counts`，只有旧 fixture 没有提供 `voxel_cells` 时才回退到 renderer-local mesh iteration。`voxel_clipmap_cell_rgba_samples`、`voxel_clipmap_cell_dominant_node_ids` 与 `voxel_clipmap_cell_dominant_rgba_samples` 则仍然留在 renderer-local mesh/material/light path，所以这一层完成的是 structural residency authority cutover，而不是最终 voxel shading authority cutover。
- `hybrid_gi/gpu_resources/execute_prepare/execute/` 与 `virtual_geometry/gpu_resources/execute_prepare/execute/` 当前都已经继续下沉成 collect-inputs / buffer / bind-group / dispatch / readback helper 子树，prepare execute 根入口只保留结构 wiring，不再混放完整执行逻辑
- `zircon_graphics::runtime::WgpuRenderFramework` 的 Hybrid GI post-render progression：当前 submit 路径已经会优先消费 renderer GPU readback 的 `cache_entries + completed_probe_ids + completed_trace_region_ids + probe_irradiance_rgb`，再回退到 `VisibilityHybridGiFeedback`，从而让下一帧 runtime snapshot 可以观察到 GPU-truth probe residency、trace schedule 与 GPU-produced irradiance 的变化；与此同时 runtime host 在接收新 extract 时也会主动裁掉已经离开场景的 stale probe、pending update 与 irradiance cache，避免旧 hierarchy 分支继续污染下一帧 probe truth
- `zircon_graphics::visibility::build_hybrid_gi_plan(...)` 的 hierarchy request/hysteresis：当前 visibility planning 除了会排除上一帧刚请求过的 probe、避免 newly resident probe 在完成 request->resident 过渡后立刻回到 `evictable_probe_ids` 之外，还会记录上一帧 active probe frontier；当 frontier 从 resident child probes 回退到 parent probe 的第一帧里，仍 resident 的 child probe 也会被额外排除出 `evictable_probe_ids`；与此同时 active resident frontier 现在也不再只收 direct child request，而是会继续把 visible nonresident descendants 放进 scene-driven request 候选，再按 trace support、ancestor trace-lineage support、hierarchy depth specificity 与 budget 裁剪；最新一层 budget 分发还会先在不同 active lineages 之间做首轮 descendant request interleave，再进入同一 lineage 的第二轮 refine，避免单条 lineage 连续吃掉全部 `probe_budget`
- `zircon_graphics::scene::SceneRenderer` 的 Hybrid GI radiance-cache lighting resolve baseline：post-process 现在会直接消费 `ViewportRenderFrame.hybrid_gi_prepare`，把 resident probe 的 `irradiance_rgb` 连同 extract 提供的 probe `position/radius` 编成 screen-projected probe buffer，并把 scheduled trace region 的 `bounds_center / bounds_radius / screen_coverage` 编成独立 screen-projected trace-region buffer；`post_process.wgsl` 现在不再只对 probe 累加结果做一份全局 trace boost，而是会把 trace region 的 screen-space support 直接并入每条 probe 的 resolve weight，让 active trace work 真正偏向附近 probe；与此同时，visibility planning 现在也会按 scheduled trace-region support 重新排序 nonresident probe request，而 GPU completion 则会在 traced radiance 上继续 gather 邻近 resident probe 的上一帧 irradiance，把 request -> update -> resolve 串成更完整的 scene-driven radiance-cache 闭环；最新的 resolve 侧编码还会沿 `parent_probe_id` ancestor chain 穿过 nonresident hierarchy gap，继续统计 resident ancestor/descendant lineage weight，并把 ancestor 覆盖到的 trace-region RT tint 预编码为 per-probe inherited lighting baseline，交给 shader 和 probe 自己的 local trace support 合并
- `zircon_graphics::runtime::HybridGiRuntimeState` 的 probe irradiance slot：当前 runtime host 不再用默认色 bootstrap resident probe；没有 GPU history 时 prepare snapshot 会导出黑值，而在 GPU readback 到达后则把 trace-region-localized、normalized-multi-region-blended `probe_irradiance_rgb` 回写到缓存里；renderer completion pass 现在还会把 resident probe 的上一帧 irradiance history 上传进 compute shader，对 resident path 执行 temporal radiance-cache update；如果本帧没有 scheduled trace work，resident probe 会保留上一帧 history、pending probe 会保持黑值，再于下一帧 build-prepare 阶段重新导出这些 GPU-produced 结果
- Repository reality note (`2026-04-18`)：`zircon_graphics` 当前真正的 façade/runtime 实现路径已经从旧文档里的 `runtime/render_framework/*` 迁移到 `runtime/render_framework/*`；`zircon_framework` 仍然保留稳定对外 API，但 `WgpuRenderFramework` 和 `submit_frame_extract/*` 才是当前 graphics crate 里的实际承载层。后续任何 M4/M5 文档都应以 `render_framework` 命名和目录为准，而不是继续把 `runtime/render_framework` 当成活跃实现真源。
- `Hybrid GI` 的 runtime resolve source closure：`HybridGiResolveRuntime` 现在已经不只缓存直接 `probe_rt_lighting_rgb`，还会把 hierarchy resolve weight、farther-ancestor irradiance continuation 与 ancestor-derived RT-lighting continuation 一起带到 post-process encode；`hybrid_gi_hierarchy_resolve_weight.rs`、`hybrid_gi_hierarchy_irradiance.rs`、`hybrid_gi_hierarchy_rt_lighting.rs` 已优先消费 runtime/GPU host source，再回退到旧的 encode-side hierarchy 扫描。
- `Hybrid GI` 的 scene-driven lineage trace support closure：runtime host 现在还会量化保留 probe / trace region scene truth，并把这条 scene-driven trace support 同时喂给 pending update 排序、GPU probe input 与 runtime resolve weight；`lineage_trace_support_q + lineage_trace_lighting_rgb` 已经沿 `prepare execute -> update_completion.wgsl -> GPU readback -> build_resolve_runtime()` 串起来，因此 nonresident hierarchy 即使还没有 resident ancestor 落地，也能继续把 trace-supported RT tint 带进 GPU source 与 runtime resolve，而不再只在 visibility request helper 或单帧 shader 启发式里存在。
- `Hybrid GI` 的 recent lineage trace-support continuation：`HybridGiRuntimeState` 现在还会维护衰减式 `recent_lineage_trace_support_q8` cache，并在 `plan_ingestion / consume_feedback / complete_gpu_updates` 更新 trace schedule 后统一刷新；这让 pending probe 排序与 `build_resolve_runtime()` 在当前 frame trace schedule 清空后，仍能继续消费最近一拍的 scene-driven hierarchy support，而不再马上退回 flat request / flat resolve weight。
- `Hybrid GI` 的 runtime-resolve -> GPU-prepare RT-lighting continuation：`SceneRendererCore::execute_runtime_prepare_passes(...)` 现在还会把 `ViewportRenderFrame.hybrid_gi_resolve_runtime` 一起送进 `HybridGiGpuResources::execute_prepare(...)`；随后 `resident_probe_inputs(...)` 会在当前 `scheduled_trace_region_ids` 没有提供 trace source 时，优先回退到 runtime hierarchy RT-lighting / runtime resolve weight，给 resident probe GPU input 补回非零的 `lineage_trace_lighting_rgb + lineage_trace_support_q`。这意味着 no-schedule frame 的 GPU prepare/readback 不再只能保 previous irradiance 而把 RT-lighting source 直接掉黑，runtime/GPU host 已确认的 hierarchy RT-lighting continuation 终于真正进入 renderer prepare 主链。
- `Hybrid GI` 的 pending-probe runtime source continuation：`HybridGiRuntimeState::build_resolve_runtime()` 现在已经不再只导出 resident probe，而会把 `pending_probes + pending_updates` 也一起并进 runtime resolve source；同时 `pending_probe_inputs(...)` 和 `resident_probe_inputs(...)` 共享同一条 `runtime_trace_source(...)` helper，因此当前 frame trace schedule 清空后，pending probe 也会继续消费 runtime hierarchy RT-lighting / resolve-weight continuation，而不再在 GPU source 里直接掉回 `[0,0,0]`。
- `Virtual Geometry` 的 deeper cluster-raster / residency continuation：`VisibilityVirtualGeometryDrawSegment` 现在已经显式携带 `lineage_depth`，并沿 `prepare -> unified indirect -> GPU submission segment readback -> virtual_geometry_indirect_args.wgsl` 继续下沉；与此同时 runtime residency completion 现在不只会在 ancestor / descendant 内部优先回收更远的 lineage distance，还会继续吃当前 `requested_pages` frontier 顺序与其他 active request lineages 的保护权重，从而让 pending uploader queue 与 eviction ordering 都围绕当前 split-merge frontier 收敛，而不再被旧 queue 顺序或输入列表顺序带偏；最新一层 visibility collapse policy 还显式区分 `requested_lineage_targets` 与 `streaming_target_lineage_targets`，因此 budget 真正塌回 coarse frontier 时，系统只会持续保热 request 自己那条恢复路径和 current streaming target 仍然 relevant 的 lineage，而不会再把 unrelated sibling subtree 一起钉住；与此同时 `pending_page_requests` 的 frontier rank 也已经继续压进 unified indirect draw contract、GPU submission segment buffer 与 `virtual_geometry_indirect_args.wgsl` 的真实 cluster-raster trim，因此较晚 request rank 现在不仅上传更晚，也会提交更收缩的 indirect args 并产生更窄的最终离屏 raster 覆盖。
- `Virtual Geometry` 的 submission-index GPU args continuation：`VirtualGeometryClusterRasterDraw.submission_index` 现在已经不只用于 prepare-owned draw 排序、draw-ref 排序和最终 `MeshDraw.indirect_args_offset` 顺序，而是会继续编码进 shared indirect segment payload，并由 `virtual_geometry_indirect_args.wgsl` 真实改变 GPU-generated `first_index / index_count`；因此即使可见实体自己的 `page_id / submission_slot / state / frontier_rank / lod_level / lineage_depth` 保持不变，只要 surrounding authoritative segment 让它的 `submission_index` 发生变化，最终 GPU args 和离屏 cluster-raster coverage 也会跟着变化，不再把这层 authority 留在 CPU submission 排序或 readback 解释层。
- `Virtual Geometry` 的 same-segment draw-ref compaction continuation：`virtual_geometry_indirect_args.wgsl` 现在还会直接扫描 shared `draw_ref_buffer`，计算当前 draw-ref 在同一 visibility-owned segment 下的真实 compaction rank，并把这条 rank 映射到新的 `first_index / index_count`；因此同一 segment 下 later primitive draw-ref 已经不再只复用 segment-level `first_index`，真正的 GPU-generated args source 现在开始消费 shared args 自己的 compaction truth。
- `Virtual Geometry` 的 pending submission-layout authority：shared indirect build 现在还会先走一层纯 CPU `build_shared_indirect_args_layout(...)`，显式导出 `pending_draw_submission_orders`，哪怕多个 pending draw 因为 `(mesh_index_count, segment_key)` compaction 落到同一条 indirect args offset，也不会再把 submission 顺序只剩给 offset 反推；随后 `build_mesh_draws(...)` 会优先按这条 direct submission order、再按 offset / original_index 稳定重排最终 `MeshDraw` 列表，因此 renderer 末端对 compacted args buffer 的依赖已经从“用 offset 重建第一排序真值”收窄成“用 direct layout authority 排第一、offset 只做 secondary stability”。
- `Virtual Geometry` 的 GPU submission-token source：`virtual_geometry_indirect_args.wgsl` 现在会在生成真实 indirect args 的同一 compute pass 里同时写出 `submission_debug_buffer` 与真实 `IndexedIndirectArgs.first_instance` token，把高 16 位 `submission_index` 和低 16 位 draw-ref compaction rank 一起压进 GPU-generated args source；`request_device(...)` 也会在 backend 支持时显式请求 `wgpu::Features::INDIRECT_FIRST_INSTANCE`，因此 visibility-owned submission truth 已经不再只停在 readback/debug buffer，而是直接进入真实 indirect execution。
- `Virtual Geometry` 的 repeated draw-ref GPU args authority：shared indirect layout 现在不再把同一 visibility-owned segment 下的重复 primitive draw 机械折叠到一条 args record；它会按 `(mesh_index_count, mesh_signature, segment_key)` 组内 occurrence 保留独立 draw-ref / args slot，因此 repeated primitive submission 不再主要依赖 CPU `MeshDraw` 重排残留来区分，而会把 per-draw authority 继续下沉到真实 GPU args offset / `first_instance` token。最新一层 authoritative compaction key 也已经把 `mesh_signature` 纳入 group key，而不再只在排序 key 里认它；所以当 authoritative draw-ref 列表里更早的 primitive 暂时不属于 drawable subset 时，surviving later primitive 也不会再被错误重映射回更早的 args slot / draw-ref rank。
- `Virtual Geometry` 的 UV-aware primitive-order residue closure：`GpuMeshResource.indirect_order_signature` 现在不再只哈希 `position + indices`，而是会继续覆盖 `position + normal + uv + indices`；因此 same-segment / same-index-count 的 overlapping primitive 即使只在 texcoord 或 normal 上不同，也不会再因为 glTF primitive 枚举顺序变化而让 repeated primitive compaction 改写最终 GPU-generated args / cluster-raster 输出。
- `Virtual Geometry` 的 fallback synthesize authority continuation：`VirtualGeometryPrepareFrame::unified_indirect_draws()` 现在还会在 synthesized fallback path 里先过滤 `Missing` clusters，并把“已有 cluster truth 但全部 `Missing`” 的 entity 直接视为 authoritative no-draw；与此同时 surviving fallback cluster 不再在过滤 `Missing` siblings 后被重新编号，而会继续保留 entity-local `cluster_start_ordinal / cluster_total_count`，因此 GPU-generated indirect args 不会再把 surviving slice 扩回 coarse full-mesh coverage。
- `Virtual Geometry` 的 deeper hot-frontier eviction continuation：runtime eviction 现在不仅会在 colder nearer descendant 与 hotter farther descendant 之间保热 hotter branch，还会让 hot later active-request lineage 压过 colder earlier lineage；如果同一 reconnecting lineage 上同时有多个 hot descendants，排序还会继续优先保留 deepest hot frontier page，而不再沿用普通 descendant 的 farther-distance-first 冷页启发式。
- `Virtual Geometry` 的 completed-descendant hot-frontier continuation：GPU completion 现在会在 `complete_gpu_uploads_with_replacements(...)` promotion 阶段就把当前 hot frontier ancestor / replaced-page truth 继承给 newly completed descendant page，因此 final page-table truth 对齐之后，下一帧 prepare recycle 不会再把刚接住 frontier 的 deeper descendant 当成 cold resident page 立刻回收。
- `Virtual Geometry` 的 deeper uploader / page-table continuation：`VirtualGeometryPrepareRequest` 现在也会显式携带 `frontier_rank`，`GpuPendingRequestInput` 与 `uploader.wgsl` 会按这条字段选取当前真正要完成的 pending page，而不再只按 pending input buffer 顺序线性消耗；GPU readback 里的 `completed_page_replacements(page_id, recycled_page_id)` 现在也已经继续进入 runtime host 与 `RenderStats.last_virtual_geometry_replaced_page_count`，让 completion / stats 主链开始直接消费 replacement truth；即使 request 没有显式 `recycled_page_id`，uploader 在隐式复用 occupied evictable slot 时也会从当前 page-table owner 生成真实 replacement readback，而不再让 runtime 只能靠 page-table aliasing 推断 fallback recycle；最新一层 uploader 还会校验显式 `assigned_slot + recycled_page_id` contract 与当前 GPU page-table 是否仍然一致，slot owner 已经漂移时会跳过 stale request，而不会继续污染 page-table completion；进一步地，prepare 现在还会为 `assigned_slot == None` 的 later request 保留 frontier-aware `recycled_page_id` 偏好，而 uploader fallback 会先尝试这条 preferred recycled page，并跳过本帧已被更早 completion 占用的 evictable slot，避免 stale request 被跳过后退回 raw slot 顺序再次回收错误 lineage；当前 submit/runtime host 还会在 record 阶段只承认最终 `page_table_entries` 真正保留下来的 completed page，并且 replacement 只会按 confirmed slot 的 previous owner、且该 owner 真正从 final page-table 消失时计数，因此 stale replacement id 或仍然 resident 的旧 owner 已经不会再污染 pending clear、completion stats 或 replacement pressure；在当前 host/runtime 主链里也没有再发现新的 raw completion side-channel 继续绕开 final page-table truth；因此 GPU uploader / page-table completion 已经开始和 unified-indirect / cluster-raster path 共用同一份 frontier truth，这为下一层 split-merge frontier policy / residency-manager cascade 提供了显式 request contract。
- `zircon_graphics::scene::SceneRenderer` 的真实 M4 runtime path：当前已经会为 `RenderFramework` 路径建立 `final_color / scene_color / bloom / gbuffer_albedo / normal / ambient_occlusion / depth / cluster_buffer` 中间资源，并按 feature 集真实分支 forward 与 deferred；forward 继续执行 mesh shader 直写 scene color，deferred 则执行 preview background、GBuffer、fullscreen deferred lighting、transparent 补绘、particle pass、bloom extract、post composite、history resolve 与 overlay
- `zircon_graphics::scene::DeferredSceneResources`：当前已经真正持有 deferred geometry 和 deferred lighting 两条 GPU pipeline，并且把 opaque 材质解码固定在 renderer 内部，而不是让 deferred 继续执行项目 fragment shader
- `zircon_graphics::runtime::offline_bake_frame(...)`：当前已经能从 extract 的方向光和几何体快照生成 `RenderBakedLightingExtract + Vec<RenderReflectionProbeSnapshot>`，并直接回灌到同一帧 runtime 数据路径
- `zircon_graphics::scene::SceneFrameHistoryTextures`：当前已经真正持有 `scene color` 与 `ambient occlusion` 两条 history texture，并在 viewport history handle 轮换或销毁时由 renderer 回收
- `zircon_graphics` 的 M4 integration renders：当前已经有离屏回归证明 history resolve 会保留上一帧颜色、SSAO 会让同一 scene 变暗、clustered lighting 会给同一 scene 带来可测量的 tile lighting tint、bloom 会扩散高亮邻域像素、color grading 会改变通道偏色、offline bake 输出会改变最终画面、particle billboard 会在 transparent stage 增加可测量热像素，而且 built-in deferred 会稳定改走 `GBuffer material decode -> deferred lighting`，与 forward project shader 路径出现可测量差异
- `zircon_graphics` 的 M5 capability-slot 回归：当前已经有单测证明默认 Forward+ 不会误带入 `VirtualGeometry / GlobalIllumination`，显式 opt-in 时会编译出新 pass 与 GI history slot，而 headless `wgpu` server 现在会把当前非 RT baseline 报告为可用，并在带 payload 的提交里写回真实 VG/GI stats
- `zircon_graphics` 的 Hybrid GI GPU update 回归：当前已经有单测证明 renderer compute pass 会稳定回传 `probe_irradiance_rgb`，而且 probe/trace 场景元数据变化时 readback 会变化，只改变 previous irradiance history 也会改变 resident probe readback；没有 scheduled trace work 时 resident probe 会保持历史、pending probe 会输出黑值，而靠近 scheduled trace region 的 probe 会得到更强的 irradiance；多个 trace region 同时命中同一 probe 时，结果现在会保持在单 region 亮度带内而不是 additive saturation；trace region 显式提供 `rt_lighting_rgb` 时，这个 override 也会直接偏置 GPU readback；同一 probe/trace 布局下，改变 scene directional light 的 tint 或 intensity 都会改变 GPU readback，runtime host 会把这些结果写回下一帧 prepare snapshot，而 newly resident probe 还会被额外保护一帧，避免 cache residency 刚完成就被立刻驱逐
- `zircon_graphics` 的 Hybrid GI GPU hierarchy continuation 回归：当前已经有单测证明 pending probe 即使隔着 nonresident hierarchy gap，也会偏向最近 resident ancestor 的 radiance，而且当本地 tracing budget 只执行 neutral local region 时，pending probe 仍然能沿 ancestor 继承到更暖的 RT-lighting tint；最新一层回归还证明当最近 resident parent 偏冷、但更远 resident ancestor 保留更暖 radiance / RT tint 时，这些 multi-ancestor lineage continuation 也会进入 GPU readback，说明 hierarchy-aware completion 已经不再只停在 resolve 侧
- `zircon_graphics` 的 Hybrid GI descendant request frontier 回归：当前已经有单测证明 active resident probe 不再只请求 direct child；当更深 descendant 对 scheduled trace region 的支持更强时，visibility planning 会直接把 descendant 选进 `requested_probe_ids`，而且当 trace support 打平或主要落在 ancestor chain 上时也会继续偏向更深 descendant，说明 scene-driven screen-probe hierarchy 已经前推到 request 层
- `zircon_graphics` 的 Hybrid GI primary-lineage gather / lineage budgeting 回归：当前已经有单测证明 pending probe 在只有 hierarchy lineage、没有 spatial overlap 时也会继承 primary resident ancestor 的 radiance，同时多个 active lineages 竞争有限 `probe_budget` 时，visibility planning 会先给每条 lineage 分到首轮 descendant request，而不是让同一 lineage 连续吞掉多个 request 槽位
- `zircon_graphics` 的 folder-backed helper compile closure：`scene_renderer_core_new`、`hybrid_gi::gpu_resources::new` 与 `virtual_geometry::gpu_resources` 的 nested helper module 现在统一收口成 subtree-scoped `pub(in crate::scene::scene_renderer::...)` 可见性，避免 sibling helper 在模块化拆分后再次因为 private re-export 路径被截断，同时没有把这些内部 helper 抬升成 crate 对外 API
- `zircon_graphics` 的 Hybrid GI runtime bootstrap 回归：当前已经有单测证明没有 GPU history 的 resident probe 不再带 host-side 默认 irradiance，而是一律以黑值等待 GPU radiance-cache output 覆盖，避免 runtime 主链继续伪造 probe 光照数据
- `zircon_graphics` 的 Hybrid GI resolve 离屏回归：当前已经有离屏测试证明 resident probe 会让 probe 覆盖区域变亮、不同 `irradiance_rgb` 会把对应区域推向不同颜色通道、probe 屏幕位置会改变哪一侧获得更多间接光、scheduled trace region 的屏幕位置会改变哪一侧获得更强的 GI boost，而且当两个 probe 同时覆盖同一区域时，scheduled trace work 现在还会真实偏向附近 probe 的颜色贡献；visibility planning 现在还支持 `parent_probe_id` 驱动的最小 hierarchy frontier、merge-back child-probe hysteresis，而 GPU completion 也已经会让 direct parent/child 关系真实改变 radiance-cache gather；最新的离屏回归还证明 resident child probe 即使通过 nonresident hierarchy gap 才连到 resident ancestor，resolve 结果也会继续变化，同时 child probe 还会继承 ancestor trace-region 的 RT tint；再往下一层，post-process probe payload 现在还会把 “beyond-nearest resident ancestor” 的 irradiance continuation 一并编码进 shader resolve，因此更远 resident ancestor 的 radiance 也会真实改变最终 GI color，而不再只停在 pending update/readback 侧；当前更远 resident ancestor 的 budget/support 还会进一步抬升 child probe 的最终 resolve 强度，说明 `runtime prepare -> GPU resource -> shader resolve` 正在收拢成更完整的 screen-probe 空间闭环
- `zircon_graphics` 的 Hybrid GI requested-lineage irradiance runtime-source continuation：当当前 frame 没有 active trace schedule、也拿不到新的 hierarchy gather 时，runtime host 现在不再只能把 pending probe 丢回黑值；如果 probe 自己已经带有上一拍 GPU 产出的 `probe_irradiance_rgb`，或者 parent chain 上仍然有带历史 irradiance 的 nonresident ancestor，而且 scene-driven requested-lineage support 仍然有效，`build_resolve_runtime()` 就会把这条 probe/ancestor runtime irradiance 重新编码进 `probe_hierarchy_irradiance_rgb_and_weight`，让 `runtime -> GPU prepare` 主链在 no-schedule frame 里继续保住 requested lineage 的 radiance-cache source，而不只剩 trace-lighting continuation。
- `zircon_graphics` 的 Hybrid GI requested-lineage RT runtime-source continuation：当前同一条 no-schedule runtime-source 闭环已经继续扩到 RT hybrid-lighting；当 pending/nonresident probe 当前拿不到 resident hierarchy RT gather，但 runtime host 仍然持有 probe 自身或 nonresident ancestor 的 `probe_rt_lighting_rgb` 且 requested-lineage support 未失效时，`build_resolve_runtime()` 现在会把这条 lineage RT history 重新编码进 `probe_hierarchy_rt_lighting_rgb_and_weight`，让 `runtime_trace_source(...)` 与 GPU prepare/readback 不再一起掉回黑值，而是继续消费同一份 scene-driven requested-lineage RT source。
- `zircon_graphics` 的 Hybrid GI scheduled/runtime trace-source blending：当前 `pending_probe_inputs(...)` 与 `resident_probe_inputs(...)` 已经不再使用 “只要 scheduled trace lighting 非零就完全覆盖 runtime hierarchy source” 的旧二选一逻辑；`merge_trace_sources(...)` 会把 current-frame scheduled trace tint 与 runtime-host hierarchy RT continuation 按 support 收束到同一份 `lineage_trace_lighting_rgb + lineage_trace_support_q` 输入，因此 current-schedule frame 现在也会继续消费 runtime hierarchy history，而不是只在 no-schedule frame 才保有 scene-driven lineage truth。
- `zircon_graphics` 的 Hybrid GI feedback completion-budget dedup：`complete_pending_probes(...)` 现在会先按输入顺序去重 `requested_probe_ids` 再应用 `probe_budget`，因此 duplicate feedback request 不会再白白吃掉唯一 eviction 并把后面的 unique pending probe 截死在 runtime queue。
- `zircon_graphics` 的 Hybrid GI scheduled trace-region dedup closure：runtime host 现在会在 `ingest_plan / consume_feedback / complete_gpu_updates` 三条入口统一先去重 `scheduled_trace_region_ids` 再刷新 lineage support，因此 duplicate trace-region id 不会再重复抬高 pending probe 排序、recent scene support 或 runtime resolve / RT continuation 权重。
- `zircon_graphics` 的 Virtual Geometry draw-level submission records：当前 repeated primitive / same-segment compaction 场景下，renderer last-state 已经不再只暴露 coarse `(entity, page_id)` submission truth；`read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 会继续基于真实 GPU `submission_tokens` 输出 `(entity, page_id, submission_index, draw_ref_rank)`，因此 unified-indirect / GPU-generated args 的 finer-grained authority 已经进一步下沉到 renderer 自身的 submission record 观测层，而不再只停在 buffer/readback 侧。
- `zircon_graphics` 的 Virtual Geometry direct submission token records：在上层 last-state helper 之外，renderer 自己现在也开始直接保存 repeated primitive / same-segment compaction 的 draw-level submission truth；`MeshDraw` 会携带 `VirtualGeometrySubmissionDetail`，`virtual_geometry_indirect_stats(...)` 与 runtime-output store 会把 `(entity, page_id, submission_index, draw_ref_rank)` 直接写进 `SceneRenderer` last-state，因此 draw-level submission records 已经不再依赖 GPU `submission_buffer` 仍然存在，coarse submission order 也和这份 direct token truth 收束到同一来源。
- `zircon_graphics` 的 Virtual Geometry actual-execution source：`virtual_geometry_indirect_stats(...)` 现在不再把 execution subset 固化在 `build_mesh_draws(...)` 的 CPU build order，而是会按真实 scene-pass 执行顺序生成 `indirect_execution_buffer` 与更深一层的 `indirect_execution_records_buffer`；因此 `Deferred` 路径下 opaque/transparent 分离后的实际 submission 次序也开始进入 renderer last-state，而不再继续停在 unified-indirect build order。与此同时 execution-record buffer 还会把 `(draw_ref_index, entity, page_id, submission_index, draw_ref_rank)` 一起压成独立 GPU source，因此 `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()` 在 CPU submission records、submission token buffer、shared indirect args/draw-ref/segment buffers 全部缺失时，仍然可以直接回读真实 execution truth，不再退回空集；同一份 execution-record truth 还会在 dedicated execution-index buffer 缺失时继续恢复真实 submitted `draw_ref_index` 次序，从而把 renderer-side submission observability 更完整地收束到一份 actual-execution contract。最新一层则继续把这份 truth 压成 execution-side summary，并沿 `SceneRenderer` last-state -> `RenderStats` 主链稳定暴露 `execution_segment/page/state/repeated_draw`，让 façade 也能直接消费真实 execution subset，而不再只能依赖 test-only readback helper；shared indirect layout 现在还会直接产出 `pending_draw_submission_plan`，因此 `build_mesh_draws(...)` 自己也不再在 renderer 末端重建一套 CPU sort key，而是直接消费 visibility-owned authoritative submission plan。
- `zircon_graphics` 的 Virtual Geometry cluster-streaming / indirect raster 离屏回归：当前已经有离屏测试证明相同 entity 在 `PendingUpload` 与 `Resident` cluster 状态下会得到不同 fallback raster 输出与覆盖面积，不同 `visible_cluster_id` 会把 fallback 压到不同屏幕区域，prepare 显式覆盖的 `cluster_draw_segments.cluster_ordinal` 也会直接改变最终栅格区域，不同 `resident_slot` 现在也会改变 resident fallback raster 输出，而且显式 prepare draw segments 即使共享同一 page/slot 也会继续保持独立 indirect submission；新的 visibility/test 还证明 hierarchy 已经会在 children 未 resident 时保留 coarse parent、在 grandchildren 未 resident 时保留 resident children，同时 request 继续追更细 frontier，而且当 refined clusters 落在同一 resident page 上时，visibility 也会继续按不同 parent lineage 保留独立 draw-segment 边界，再由 runtime prepare 原样传给 unified indirect path；最新一层 visibility cascade 还证明，当上一帧 frontier 已经下探到 resident descendants，而当前帧因为中间 ancestor page 掉 resident 导致 frontier 多级塌回 coarse parent 时，planning 会优先请求缺失 ancestor page，并把上一帧活跃的 resident descendants 从首轮 `evictable_pages` 里保护出去；GPU readback 测试则继续证明 `cluster_span_count=1/2` 会生成不同的 indirect args `(first_index, index_count)`，而且不同 resident page 即使暂时共享同一 slot 也会保持独立 indirect draw，并且现在还会进一步生成不同的 GPU indirect args / raster 子范围；最新一层透明重叠离屏回归还证明当 prepare `submission_slot` 互换时，中心像素主导色也会跟着翻转，说明真实 `MeshDraw -> draw_indexed_indirect(...)` 执行顺序已经开始直接消费 visibility-owned indirect authority，而不再只在 buffer/readback 层排序；进一步地，当只有 surrounding authoritative segment 改变顺序、而可见实体自己的 `page/slot/state/frontier/lod/lineage` 保持不变时，新的回归现在也证明 `submission_index` 自身已经会改变真实 GPU-generated indirect args 和最终离屏覆盖，因此更深一层的 cluster-raster submission ownership 已经不再依赖“CPU mesh draw 顺序先变，像素才跟着变”的浅层路径；与此同时没有显式 `cluster_draw_segments` 的 full-mesh fallback 现在也不再退回 renderer 自造的 `page 0 / slot 0 / resident` key，而会直接继承 `visible_clusters + pending_page_requests` 推导出的 page/slot/state truth，并在同一 entity 的多个 visible clusters 里选择最 authoritative 的 cluster；最新一层 downshift 还把这批 fallback cluster slices 继续前移到 `VirtualGeometryPrepareFrame::unified_indirect_draws()` 自身，因此 missing-segment entity 也会先生成 prepare-owned fallback indirect draws，再交给 `build_virtual_geometry_cluster_raster_draws(...)`、shared `segment_buffer / draw_ref_buffer / indirect args` 与真实 submission 一起消费，而不再主要依赖 renderer 末端的 CPU fallback key 扩展；再往下一层，mesh-build 自身也已经不再平行维护 fallback segment bookkeeping，而是明确只消费 `virtual_geometry_cluster_draws` 这份 prepare-owned cluster-raster truth，并且会把 “没有 cluster-raster truth” 视为 authoritative no-draw contract，因此 explicit `Missing` segment entity 不会再在 renderer 末端被复活成 full-mesh fallback draw；与此同时 `RenderStats` 现在也已经把 compacted `indirect_args_count` 显式暴露出来，因此 façade 不只知道 draw/segment，还能直接看到 unified-indirect compaction 是否真实发生；与此同时 split-merge 稳定层已经同时覆盖 coarse-parent hold、merge-back child-page hold 与 multi-level frontier collapse hold，避免 parent/child/deeper descendant page 在 frontier 切换当帧立刻被回收；而最新一层 runtime eviction 还会在 reconnect missing ancestor 时优先保留 recently-hot 的 farther descendant，不再只因为 lineage distance 更远就先踢掉当前最热 branch；GPU uploader 现在已经会拒绝超出 streaming byte budget 的大页，并跳过 oversized request 去完成后续能装入预算的小页，还会优先消耗 prepare 提供的 free/future `available_slots` 再复用 evictable resident slot，并在同一帧把 post-upload page table snapshot 一起读回；runtime host 现在也会把这份 `page_table_entries` 当成 residency truth 回灌 `VirtualGeometryRuntimeState`，让 page eviction、slot reassignment 与下一帧 `available_slots` 级联都跟随 GPU 页表，而当前 slot recycling 还额外只信当前帧 `evictable_pages` 真值，不会再因为 runtime 内部旧状态把被 merge-back / cascade 保护撤回的 resident page 误回收；`RenderStats` 也已经会把 `completed_page_count`、`indirect_args_count` 与 prepare-owned `indirect_segment_count` 一起暴露出来，说明 prepare snapshot 的 streaming 状态、size budget、slot ownership、page-table snapshot、runtime residency cascade、streaming-aware cluster frontier、segment contract 与 indirect raster baseline 都已经进入真实 draw submission
- `zircon_graphics` 的 Virtual Geometry completion-budget cascade closure：GPU completion 路径现在不再在 stale slot assignment 通过 validation 之前就被 `page_budget` 提前截断，而 feedback completion 也会先按输入顺序去重 `requested_pages` 再应用 budget；因此 later unique page completion 不会再被 leading stale/duplicate completion 吃掉，runtime residency/page-table cascade 终于收束到真正有效的 completion truth。
- `zircon_graphics` 的 Virtual Geometry first-unique GPU completion truth：`complete_gpu_uploads_with_replacements(...)` 现在会先按输入顺序去重 duplicate `(page_id, slot)` completion，再处理 replacement 与 resident promotion，因此同一个 page 的后续 duplicate completion 不会再重写已确认 slot，也不会再把后面的 unique page completion 卡死在 runtime residency cascade。
- `zircon_graphics` 的 Virtual Geometry normalized page-table truth：`normalized_page_table_entries(...)` 现在会从尾到头反扫 raw `page_table_entries`，同时按 `page_id` 与 `slot` 去重，再按 slot 重新排序成 final table truth；因此同一 page 的 later stale duplicate 不会再抹掉它更早仍然有效的 resident slot，而真正 surviving 的 slot reassignment 也会继续保留到最终 runtime truth。`apply_gpu_page_table_entries(...)` 与 `confirmed_virtual_geometry_completion(...)` 现在都消费同一份 normalized table，所以 runtime apply、completion stats、replacement inference 与下一帧 frontier recycle 不会再各自解释出不同的 page-table 结论；与此同时 same pending page 的 raw duplicate table entry 也不会再把 `completed_page_replacements` 重复计数到 façade stats。
- `zircon_graphics` 的 Virtual Geometry multi-frame hot-frontier cooling cascade：runtime host 现在会把 confirmed hot frontier 下沉成带 `frames_remaining` 的 `recent_hot_resident_pages` cooling 窗口，而不再只保留“上一拍 recent set”；`refresh_hot_resident_pages(...)` 会在每次 feedback 前先衰减旧窗口，再把上一拍 `current_hot_resident_pages` 重新注入固定 cooling budget，因此 `ordered_evictable_pages_for_target(...)`、`complete_pending_pages(...)`、`complete_gpu_uploads_with_replacements(...)`、`apply_gpu_page_table_entries(...)` 会继续共享 `current + cooling-window` 的同一份 lineage-aware hot truth。当前 cooling 预算已经能跨两次 cooling feedback frame 保住更深的 confirmed descendant branch，同时仍然会在预算耗尽后干净退回 colder-depth ordering，不会把 split-merge frontier 变成无限期热偏置。
- `zircon_graphics` 的 Virtual Geometry feedback-completion frontier carry-forward：`complete_pending_pages(...)` 现在也会在 promote 前读取 `page_or_lineage_is_hot(page_id)`，并把这份 `current + cooling-window` confirmed frontier truth 写回 newly completed page；因此 no-GPU-completion 的 feedback 分支不再落后于 `complete_gpu_uploads_with_replacements(...)`，reconnected ancestor 在 hotter descendant 离开 residency 之后也不会立刻掉回冷页并在下一次 recycle plan 中被过早回收。
- `zircon_graphics` 的 Virtual Geometry live-extract frontier pruning：`register_extract(...)` 现在会像处理 `current_hot_resident_pages` 一样同步裁剪 `recent_hot_resident_pages` cooling 窗口，因此已经离开 live extract 的旧 frontier page 不会把陈旧的 hot TTL 带过 extraction gap 再回流到后续 recycle plan；当同一 page id 重新出现时，它只会继承当前 live runtime 真值，而不会因为过去某一帧的 cooling residue 继续被误保护。
- `zircon_graphics` 的 Virtual Geometry confirmed page-table reconnect frontier merge：`apply_gpu_page_table_entries(...)` 现在会把 “同 slot 替换 hot page” 和 “沿 lineage 继承 hot frontier” 两种来源拆开处理，只有仍然存活在 final page table 里的 confirmed hot pages 才能继续通过 ancestor/descendant lineage 把热度传给 reconnected page；已经从 authoritative page table 消失的 hot descendant 不会再在 runtime apply 之后继续偏置下一拍 recycle/hold/reconnect。与此同时 `runtime/virtual_geometry/test_accessors.rs::apply_evictions(...)` 也已经改为走同一条 `evict_page(...)` 路径，避免测试辅助入口绕开真实 residency merge 语义并残留旧 hot frontier truth。
- `zircon_graphics` 的 Virtual Geometry first-unique request-order closure：`plan_ingestion(...)` 现在会把 `requested_pages` 的第一次出现位置固定成 `current_requested_page_order` 真值，而不是让 later duplicate 覆盖它；因此 pending upload frontier、slot recycle 与 eviction lineage priority 不会再被重复 request 噪声反向改序。
- `zircon_graphics` 的 Hybrid GI first-unique GPU cache truth：`apply_gpu_cache_entries(...)` 现在会先按输入顺序去重 duplicate `(probe_id, slot)` cache entry，再更新 resident probe truth，因此同一 probe 的后续 duplicate cache entry 不会再迁移已确认 resident slot，也不会再把后续 unique probe 挤出最终 runtime cache snapshot。
- `zircon_runtime` 的 Hybrid GI scene-prepare voxel authority-color seam：`HybridGiScenePrepareResourcesSnapshot` 现在除了 `voxel_clipmap_cell_dominant_node_ids` 之外，还会把 `voxel_clipmap_cell_dominant_rgba_samples` 一起压回 renderer readback，因此同一 coarse voxel cell 在有重叠 contributor 时，框架已经能同时分辨 aggregate cell sample 与 dominant contributor 自身的颜色/能量真值，而不必再把 authority 只压成一个 node id。
- `zircon_runtime` 的 Hybrid GI scene-prepare voxel residency-count seam：`HybridGiVoxelSceneState` 现在已经在 runtime host 内部按 resident clipmap 派生固定 `64` 项 `voxel_cells` occupancy payload，并通过 `HybridGiRuntimeState::build_scene_prepare_frame()` 下发到 renderer；因此 `voxel_clipmap_occupancy_masks` 与 `voxel_clipmap_cell_occupancy_counts` 已经不再默认依赖 renderer-local `scene_meshes` 重算，而是优先服从 runtime-owned scene representation truth。只有迁移期 fixture 没有提供 `voxel_cells` 时，renderer 才会临时保留 mesh-derived fallback。
- `zircon_graphics::visibility` 的 support-layer 编译边界：`culling/` 与 `planning/` 的 helper 现在通过显式模块路径暴露给 `VisibilityContext`，`is_mesh_visible(...)` 也稳定改用 `transform_point3(...)`，从而恢复 `cargo test -p zircon_graphics --lib --locked`
- `zircon_editor` 的 Slint viewport controller 通过 `RenderFramework` 创建/重建 viewport，并从 capture 拉回最新帧
- `zircon_editor` 的 shared viewport toolbar pointer route 通过同一 runtime dispatch 路径触发 typed `ViewportCommand`
- `zircon_app` 的 runtime presenter bridge 通过 `RenderFramework` 管理 viewport 生命周期并返回最新 captured frame
- `zircon_editor` 的 shared pointer callback source-location tests 现在接受 `app.rs` 与 `app/callback_wiring.rs` 双路径，从而保持 root entry file 精简和 pointer wiring 模块化并存
- editor/runtime 的源码边界测试会阻止 `wgpu`、`RuntimePreviewRenderer`、`SharedTextureRenderService` 等旧上层消费路径重新回流
- 受影响 crate 当前已通过：
  - `cargo test -p zircon_graphics --locked render_framework_bridge`
  - `cargo test -p zircon_scene --locked`
  - `cargo test -p zircon_graphics pipeline_compile --locked`
  - `cargo test -p zircon_graphics compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features --locked`
  - `cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked`
  - `cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked`
  - `cargo test -p zircon_graphics virtual_geometry_gpu --locked`
  - `cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_respects_streaming_bytes_even_with_evictable_pages --locked`
  - `cargo check -p zircon_graphics --lib --locked --target-dir target/codex-shared-b`
  - `cargo test -p zircon_graphics --lib project_render --locked --target-dir target/codex-shared-b`
  - `cargo check -p zircon_scene --tests --locked --target-dir target/codex-shared-b`
  - `cargo test -p zircon_scene --test render_frame_extract --locked --target-dir target/codex-shared-b`
- 同一轮继续补齐后也已通过：
  - `cargo test -p zircon_scene --test viewport_packet --locked --target-dir target/codex-shared-b`
  - `cargo check --workspace --locked --target-dir target/codex-shared-b`
- 扩展验证队列仍包括：
  - `cargo test -p zircon_graphics confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization --locked`
  - `cargo test -p zircon_graphics confirmed_virtual_geometry_completion_normalizes_reassigned_page_table_truth_before_runtime_apply --locked`
  - `cargo test -p zircon_graphics virtual_geometry_runtime_state_keeps_reassigned_page_table_owner_in_next_frontier_recycle_plan --locked`
  - `cargo test -p zircon_graphics virtual_geometry_gpu_uploader_readback_skips_oversized_requests_and_completes_ones_that_fit --locked`
  - `cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked`
  - `cargo test -p zircon_graphics virtual_geometry_prepare_streaming_state_changes_fallback_raster_output --locked`
  - `cargo test -p zircon_graphics virtual_geometry --locked`
  - `cargo test -p zircon_graphics visibility_context_builds_hybrid_gi_probe_and_trace_plan --locked`
  - `cargo test -p zircon_graphics visibility_context_with_history_tracks_hybrid_gi_requested_probes --locked`
  - `cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces --locked`
  - `cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_previous_irradiance_changes --locked`
  - `cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_normalizes_multi_region_radiance_instead_of_additive_saturation --locked`
  - `cargo test -p zircon_graphics hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule --locked`
  - `cargo test -p zircon_graphics hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots --locked`
  - `cargo test -p zircon_graphics hybrid_gi_runtime_state_keeps_processing_later_unique_feedback_probe_completions_after_leading_duplicate_requests --locked`
  - `cargo test -p zircon_graphics hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule --locked`
  - `cargo test -p zircon_graphics hybrid_gi --locked`
  - `cargo test -p zircon_graphics hybrid_gi_resolve_localizes_indirect_light_by_probe_screen_position --locked`
  - `cargo test -p zircon_graphics hybrid_gi_resolve_uses_prepare_probe_irradiance_colors --locked`
  - `cargo test -p zircon_graphics hybrid_gi_resolve_prefers_screen_probe_irradiance_supported_by_scheduled_trace_regions --locked`
  - `cargo test -p zircon_graphics history_resolve_blends_previous_scene_color_when_enabled --locked`
  - `cargo test -p zircon_graphics ssao_quality_profile_darkens_scene_when_enabled --locked`
  - `cargo test -p zircon_graphics clustered_lighting_quality_profile_applies_runtime_tile_lighting --locked`
  - `cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked`
  - `cargo test -p zircon_graphics bloom_quality_profile_spreads_bright_pixels_when_enabled --locked`
  - `cargo test -p zircon_graphics color_grading_extract_tints_scene_after_post_process --locked`
  - `cargo test -p zircon_graphics offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering --locked`
  - `cargo test -p zircon_graphics particle_rendering_draws_billboard_sprites_in_transparent_stage --locked`
  - `cargo test -p zircon_graphics render_server_bridge --locked`
  - `cargo test -p zircon_graphics visibility --locked`
  - `cargo test -p zircon_graphics --lib --locked`
  - `cargo check -p zircon_graphics --lib --locked`
  - `cargo test -p zircon_graphics --locked render_framework_bridge`
  - `cargo test -p zircon_app --lib --locked`
  - `cargo test -p zircon_editor --lib --locked`
  - `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`

当前还没有完成的验证：

- assetized `RenderPipelineAsset` 真正接入 shader/material/feature 选择
- GPU-driven visibility 的 occlusion、真正 visibility-owned unified indirect args buffer 编码、真实 BVH 构建
- `Virtual Geometry` 的更深层 unified indirect / cluster raster / GPU-driven indirect compaction / 更完整的 split-merge hierarchy hysteresis / Nanite-like cluster raster
- `Hybrid GI` 的 screen-probe hierarchy / RT hybrid lighting



