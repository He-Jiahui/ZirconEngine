---
related_code:
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/docs/profiling.md
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/view/window/mod.rs
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/erased_render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mesh_allocator_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_pbr/src/lib.rs
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/bevy/crates/bevy_pbr/src/material.rs
  - dev/bevy/crates/bevy_pbr/src/mesh_material.rs
  - dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/bevy/crates/bevy_pbr/src/render/pbr.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/pbr_lighting.wgsl
  - dev/bevy/crates/bevy_pbr/src/deferred/deferred_lighting.wgsl
  - dev/bevy/crates/bevy_pbr/src/cluster/cluster.wgsl
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/volume.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/capability.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/history.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/virtual_geometry.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/hybrid_gi.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/advanced_provider.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/solari.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/config.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_asset.rs
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/runtime/history/is_compatible.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/graphics_debugger_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/environment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/request_graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/query_graphics_debugger_status.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/submit_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/create.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset/set_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile/set_quality_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/generation.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/fallback/create_fallback_texture.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/importer/ingest/import_cube_lut.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/mod.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_post_process_lut_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_new.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_post_process_lut_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/constants/gbuffer_material_format.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom/execute_bloom.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/construct/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/effect_lut_texture_view.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/fallback_texture_views/fallback_texture_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/neural_compute.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/requires_explicit_opt_in.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/partition_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/record_overlays.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/volume.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/capability.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/history.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/virtual_geometry.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/hybrid_gi.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/advanced_provider.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/solari.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/diagnostics/profiling/mod.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/config.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/runtime/history/is_compatible.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/graphics_debugger_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/environment.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/request_graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/query_graphics_debugger_status.rs
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture/submit_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/create_viewport/create.rs
  - zircon_runtime/src/graphics/runtime/render_framework/destroy_viewport/destroy_viewport.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_pipeline_asset/set_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/set_quality_profile/set_quality_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/generation.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom/execute_bloom.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/construct/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/effect_lut_texture_view.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/fallback_texture_views/fallback_texture_views.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/neural_compute.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/requires_explicit_opt_in.rs
  - zircon_runtime/src/graphics/pipeline/compile_options/methods.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/partition_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/record_overlays.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_history/prepare_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
plan_sources:
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-05-22 continue M10 render diagnostics and profiling bridge checklist
  - user: 2026-05-21 continue M10 default 3D PBR and light acceptance checklist
  - user: 2026-05-21 continue M10 default 2D and presentation base acceptance checklist
  - user: 2026-05-21 continue Bevy PBR material and lighting evidence mapping
  - user: 2026-05-21 continue Bevy presentation surface evidence mapping
  - user: 2026-05-21 continue Bevy render diagnostics evidence mapping
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/framework/tests.rs::render_camera_contracts_cover_viewports_and_bevy_layer_intersection
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::dynamic_resolution_scales_internal_graph_resources_without_resizing_viewport_output
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_reports_frame_history_invalidation_when_camera_moves
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_records_history_resolve_after_compatible_history_exists
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_reuses_frame_history_handle_for_compatible_submissions
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_invalidates_history_when_dynamic_render_size_changes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::ssao_executor_requires_post_process_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::clustered_lighting_executor_requires_post_process_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::bloom_extract_executor_requires_post_process_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::preview_sky_executor_requires_preview_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/tests/render_profiling.rs
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs
  - cargo test -p zircon_runtime --locked render_product_sprite
  - cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - cargo test -p zircon_runtime --locked camera_target
  - cargo test -p zircon_runtime --locked surface_targets
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_pbr_submit_reports_material_fallback_and_light_stats
  - zircon_runtime/src/asset/assets/texture/upload_support/tests.rs::rgba8_upload_readiness_accepts_layered_shapes_with_complete_payloads
  - zircon_runtime/src/asset/assets/texture/upload_support/tests.rs::rgba8_upload_readiness_accepts_complete_layered_mip_chain_payloads
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs::tests::rgba8_mip_uploads_pack_levels_and_layers_in_payload_order
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs::tests::rgba8_mip_uploads_pack_layers_inside_each_mip_level
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs::tests::rgba8_material_texture_view_keeps_current_d2_binding_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::overlay_executor_requires_overlay_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::sprite_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::mesh_executor_requires_mesh_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::deferred_gbuffer_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::deferred_lighting_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::shadow_map_executor_requires_graph_shadow_map_resource_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::shadow_map_executor_records_depth_only_pass_when_graph_resource_is_bound
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_creates_dense_transients_and_skips_sparse_reservations
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::shadow_map_pass_stays_live_as_depth_only_graph_contract
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_shadow_map_graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::plugin_render_feature_descriptors_require_explicit_executor_registration
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::particle_plugin_executor_ids_require_explicit_registration
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs::register_pipeline_asset_rejects_plugin_executor_from_descriptor_only
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs::register_pipeline_asset_accepts_plugin_executor_from_explicit_registration
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs::reload_pipeline_rejects_plugin_executor_from_descriptor_only
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs::reload_pipeline_accepts_plugin_executor_from_explicit_registration
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::execution_record_preserves_pass_debug_markers
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs::renderdoc_debug_marker_registry_covers_capture_timeline
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_records_storage_writes_without_attachment_ops
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compile_options_fallback_async_compute_passes_to_graphics_queue
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::pipeline_compile_rejects_storage_write_mode_on_read_access
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_tracks_compute_dispatch_metadata
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_audits_planned_compute_workloads_against_dispatches
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_flags_compute_workload_label_workgroup_and_extent_mismatches
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_preserves_compute_workload_metadata
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_preserves_compute_workload_from_feature_descriptor
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_rejects_compute_workload_on_non_compute_queue
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::headless_wgpu_server_falls_back_async_compute_passes_to_graphics
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - cargo test -p zircon_runtime --lib render_pass_executor_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib register_pipeline_asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib reload_pipeline --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib graph_records_storage_writes_without_attachment_ops --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib compile_options_fallback_async_compute_passes_to_graphics_queue --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib pipeline_compile_rejects_storage_write_mode_on_read_access --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib execution_record_tracks_compute_dispatch_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib compute_workload --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs::compiled_pipeline_capability_validation_reports_neural_compute_requirement
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::flagship_feature_descriptors_declare_backend_capability_requirements
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::neural_compute_builtin_slot_compiles_only_with_explicit_feature_opt_in
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::plugin_neural_compute_feature_respects_capability_opt_in_gate
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::advanced_followup_feature_slots_reserve_extract_sections_without_runtime_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::advanced_followup_builtin_slots_compile_only_with_explicit_feature_opt_in
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::sparse_texture_builtin_slot_requires_feature_and_capability_opt_in
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs::compiled_pipeline_capability_validation_reports_sparse_texture_requirement
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::renderer_data_document_accepts_neural_compute_builtin_feature_source
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::renderer_data_document_accepts_advanced_followup_builtin_feature_sources
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_rejects_neural_compute_plugin_descriptor_without_executor_registration
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_rejects_neural_compute_plugin_pipeline_when_backend_capability_is_missing
  - zircon_runtime/src/core/framework/render/backend_types.rs::capability_class_report_splits_default_advanced_and_experimental_requirements
  - zircon_runtime/src/rhi/tests/capabilities.rs::backend_caps_report_queue_classes_and_rt_support_independently
  - zircon_runtime/src/rhi_wgpu/tests.rs::wgpu_caps_fall_back_to_graphics_and_copy_without_rt
  - cargo test -p zircon_runtime --lib compiled_pipeline_capability_validation_reports_neural_compute_requirement --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - zircon_runtime/src/graphics/tests/project_render.rs::deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs::sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs::sprite_queue_stats_count_stage_batches_sprites_and_vertices
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs::compiled_scene_outputs_carry_prepared_sprite_queue_stats
  - zircon_runtime/src/core/framework/tests.rs::render_product_post_process_effect_stack_runs_before_final_composite_when_authored
  - zircon_runtime/src/core/framework/tests.rs::render_product_post_process_extended_effect_stack_settings_enable_product_node
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_records_active_approximated_and_missing_resources
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_treats_authored_lut_as_renderer_bound_resource
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_treats_bound_ssr_normal_as_available
  - zircon_runtime/src/core/framework/render/post_process/stack.rs::tests::effect_stack_ssr_declares_depth_normal_and_material_inputs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_scene_material_texture_for_ssr_roughness
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_id_uses_enabled_lookup_handle
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_id_ignores_disabled_lookup_handle
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_request_tracks_enabled_lut_without_handle
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::color_lookup_texture_layout_accepts_current_2d_strip_contract
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::color_lookup_texture_3d_layout_is_recognized_but_not_2d_bindable
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_records_invalid_lut_layout_size
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_status_accepts_2d_strip_for_current_binding
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_status_accepts_3d_lut_for_texture_3d_binding
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_status_rejects_non_2d_binding_shapes
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_clamps_channels_and_builds_3d_texture
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_ignores_common_metadata_rows
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_rejects_1d_shaper_sections
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs::tests::cube_lut_parser_rejects_out_of_range_sizes
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_decodes_cube_lut_as_linear_3d_rgba8_texture
  - zircon_runtime/src/asset/tests/assets/texture_importer.rs::importer_rejects_cube_lut_with_wrong_sample_count
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_stats_report_effect_stack_product_node_when_authored
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_stats_report_volume_effect_stack_product_node_when_authored
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::effect_stack_settings_are_encoded_into_post_process_params
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::orthographic_camera_depth_params_disable_perspective_linearization
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::camera_view_basis_is_encoded_for_post_process_normals
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/effect_lut_texture_view.rs::tests::generated_effect_lut_is_s_curve_with_stable_texture_stride
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/effect_lut_texture_view.rs::tests::generated_effect_lut_3d_is_identity_cube
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_parses_after_lut_binding_expansion
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_effect_lut_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_effect_lut_texture_3d
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_scene_depth_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_refines_projected_ssr_hits
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_scene_normal_texture_for_ssr
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_uses_lens_bokeh_depth_of_field_kernel
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_viewport_depth_fallback_shader_parses_for_gl_backends
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs::tests::viewport_depth_fallback_shader_removes_raw_depth_texture_sampling
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::depth_of_field_lens_settings_are_sanitized_for_renderer_upload
  - zircon_runtime/src/core/framework/render/post_process/volume.rs::tests::volume_stack_resolves_extended_effect_stack_settings
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::effect_stack_resource_status_detects_graph_bound_ssr_normal
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Render Product Submit

M4A makes `RenderFrameExtract` the product submit authority. `ViewportRenderFrame::from_extract` keeps a legacy scene snapshot field for compatibility with older public-runtime paths, but renderer internals read camera, mesh, light, overlay, and preview data through accessors backed by `RenderFrameExtract`.

## Bevy Render Diagnostics Evidence

Bevy's render foundation is split between the render sub-app, optional pipelined rendering, and render diagnostics. `dev/bevy/crates/bevy_render/src/lib.rs:120-208` defines `RenderPlugin`, `RenderApp`, `RenderStartup`, and `RenderSystems` for extract, prepare-assets, queue, phase sort, prepare resources, render, cleanup, and post-cleanup. `dev/bevy/crates/bevy_render/src/pipelined_rendering.rs:68-105` moves rendering to a separate thread so frame `N` rendering can overlap frame `N + 1` simulation, with a `RenderExtractApp` handling sync/extract and channels moving the render sub-app between threads.

`dev/bevy/crates/bevy_render/src/diagnostic/mod.rs:37-63` defines `RenderDiagnosticsPlugin` as the owner for render CPU/GPU elapsed time per pass and pipeline statistics. The same docs route consumers through `DiagnosticsStore`, `LogDiagnosticsPlugin`, or Tracy, and state that GPU timestamp and pipeline-statistic support is limited to Vulkan and DX12 while other backends record CPU time only.

`diagnostic/mod.rs:66-94` wires diagnostics in two worlds: the main app stores a `RenderDiagnosticsMutex` and syncs diagnostics in `PreUpdate`, while the render app initializes `DiagnosticsRecorder` and adds begin, resolve, and finish systems to the render graph. `diagnostic/mod.rs:132-192` exposes the pass API through `RecordDiagnostics::time_span`, `pass_span`, `record_f32`, and `record_u32`; the guards require explicit `end(...)`, making missing span closure visible during development.

`dev/bevy/crates/bevy_render/src/diagnostic/internal.rs:23-29` fixes query budgets and buffer sizes for timestamps and pipeline statistics. `internal.rs:83-144` rotates current, submitted, and finished frame diagnostic buffers; `internal.rs:244-285` only creates timestamp and pipeline-statistics query sets when the backend supports them. This is a true GPU diagnostic path, not just a CPU-side render stats snapshot.

Bevy also treats render-asset residency as diagnostics. `render_asset_diagnostic_plugin.rs:31-42` registers a `render_asset/<type>` diagnostic and measures `RenderAssets<A>` during `ExtractSchedule`, then reports the count in `PreUpdate`. `erased_render_asset_diagnostic_plugin.rs:35-46` does the same for erased render assets, while `mesh_allocator_diagnostic_plugin.rs:36-52` registers mesh allocator slab count, slab byte size, and allocation count.

## Zircon Diagnostic State

Zircon currently records render health through submit-owned `RenderStats`, not through a Bevy-style GPU timing recorder. `RenderStats` in `zircon_runtime/src/core/framework/render/backend_types.rs` carries submitted-frame generation, effective feature names, planned and executed render graph pass/resource/dependency counts, render graph pass debug markers, compute dispatch/storage-write evidence plus compute workload audit counts, transient graph allocation slots and byte reservations, post-process graph nodes, the effect-stack family report, LUT request/ready/fallback plus shape counters, anti-alias fallback, advanced provider reports, Solari status, queue fallback, async compute pass counts, UI command/payload/order stats, material/sprite readiness, mesh queue-preparation counters, light ready/degraded splits, and VG/HGI runtime counters.

`update_base_stats(...)` in `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs` writes the baseline stats after a successful submit from the frame submission context, compiled pipeline, renderer execution record, post-process graph, resolved effect-stack report, renderer-side LUT readiness/shape counters, anti-alias fallback, advanced/Solari reports, UI stats, material/sprite renderer stats, mesh queue-preparation stats, and shared `RenderLightReadinessReport`. `query_stats(...)` in `render_framework/query_stats/query_stats.rs:5-8` returns a clone of this stats struct to runtime diagnostics and editor/tooling consumers.

The existing debug marker and RenderDoc path is adjacent but not equivalent to Bevy render diagnostics. `debug_markers.rs` and the compiled-scene renderer label GPU events for external capture; they do not yet feed CPU/GPU pass durations, pipeline statistics, or render-asset diagnostics into `DiagnosticStore`.

RenderGraph submit counters are now mirrored into runtime diagnostics as product telemetry. `collect_runtime_diagnostics(...)` keeps the legacy `render.last_graph_executed_pass_count` path and also records stable `render.graph.*` count rows for planned passes, culled passes, queue fallbacks, resource lifetimes, sparse texture reservation lifetimes, planned resource accesses, planned dependencies, dense transient texture slots, sparse texture reservation slots, transient buffer slots, executed passes, executed resource accesses, executed dependencies, pass-level debug marker coverage, compute dispatch count, aggregate compute dispatch group volume, compute storage-write resource count, and compute workload audit counts for planned, matched, missing, mismatched, and unexpected dispatches. It also records `bytes` rows for dense transient texture reservations, dense transient buffer reservations, combined dense transient reservation pressure, and sparse virtual texture footprint. These rows expose M2/M5/M8 graph health without leaking renderer-private pass records or WGPU objects.

Capability gates follow the same neutral DTO path. `RenderStats.capabilities` stays the authoritative backend summary, and `collect_runtime_diagnostics(...)` records `render.capability.*` bool/count rows for queue class count, surface/offscreen support, async compute/copy, pipeline cache, storage buffers, indirect draw, buffer readback, raytracing/resource-indexing features, AA support, max MSAA samples, VG/HGI support, and the `neural_compute` / `sparse_texture` capability slots. `BuiltinRenderFeature::NeuralCompute` now provides a descriptor-only SRP slot that declares `RenderFeatureCapabilityRequirement::NeuralCompute` only after explicit feature opt-in, while plugin neural compute descriptors still need explicit capability opt-in before their async-compute passes compile into the graph. `BuiltinRenderFeature::SparseTexture` follows the same descriptor-first rule but declares `RenderFeatureCapabilityRequirement::SparseTexture`, so compile requires both feature opt-in and capability opt-in, and runtime activation rejects the compiled pipeline when the backend summary reports no sparse texture support. The graph bridge also records executed family counts for AA, VG, HGI, particles, transparent stage work, and async-compute passes under `render.graph.executed_*_pass_count`.

M8 follow-up feature slots use the same descriptor-first boundary. `SparseTexture`, `Particle`, `Terrain`, `Tree`, `Projector`, `Halo`, `LensFlare`, `Trail`, `Billboard`, `Tilemap`, and `TextShaping` are now built-in SRP tokens that require explicit feature opt-in and reserve one neutral extract section each. `SparseTexture` is the only slot in this group that also carries a backend capability requirement today: RHI exposes `RenderBackendCaps.supports_sparse_texture`, WGPU maps it to `false`, diagnostics mirror it as `render.capability.sparse_texture_supported`, and compiled-pipeline capability validation fails on `sparse_texture` until a backend/provider explicitly opts in. The RHI/RenderGraph resource contract now also has `TextureResidency::SparseReserved`, which preserves a sparse texture's virtual extent, layers, mips, format, and usage while keeping it out of dense transient texture aliasing. `update_base_stats(...)` copies the compiled graph's sparse reservation lifetime count, sparse reservation slot count, dense transient reservation byte totals, and sparse virtual texture bytes into `RenderStats`; `DiagnosticStore` mirrors the slot/lifetime rows as counts and the byte rows as `bytes`. Current WGPU headless creation rejects sparse reservations because backend support is false; no page table, tile upload, residency allocator, or sparse WGPU object is implied yet. The other follow-up slots intentionally compile no pass, executor, history binding, or backend capability requirement. The built-in `Particle` slot reserves `particles` extract data for authored renderer-data and profile gating only; the executable transparent particle pass still comes from a plugin descriptor plus explicit `particle.transparent` executor registration. Authoring can name these sources in renderer data before their dedicated asset/extract/renderer plans land, but enabling a slot only proves the pipeline and extract contract exists; sparse residency, particle simulation/rendering, terrain/tree rendering, projector decals, halo sprites/glow, lens flare occlusion, trails, billboard batching, tilemap variants, and text shaping remain follow-up implementations that must register real descriptors or plugin executors through this main chain.

Post-process graph diagnostics are separate from effect-stack readiness. `RenderStats.last_post_process_graph_*` keeps the product postprocess node counts and executed-node list, and `collect_runtime_diagnostics(...)` records the numeric portion as `render.post_process.graph.node_count`, `skipped_node_count`, `executed_node_count`, and `final_composite_present`. That keeps graph scale and final composition presence visible without pushing string node lists into `DiagnosticStore`.

Effect-stack readiness now distinguishes authored-resource gaps from graph-bound renderer resources. `update_base_stats(...)` derives a `RenderPostProcessEffectStackResourceStatus` from the effective `PostProcessPassGraph`, so SSR no longer reports `effect-stack.ssr.normal` when the active `EffectStack` node declares `gbuffer-normal` as an input. The concrete postprocess resources keep that renderer detail private: binding 14 receives the graph normal texture, binding 16 receives `gbuffer-material` only when the active compiled graph has an unculled writer for it, and `PostProcessDepthSamplingMode` keeps raw scene-depth sampling on supported backends while compiling a viewport-depth fallback shader for GL/WebGL/ANGLE. DoF lens controls are now part of the same submit-owned uniform path: `RenderDepthOfFieldSettings` carries focus range, focal length, bokeh blade count, and bokeh rotation; `PostProcessParams.effect_dof_lens` uploads sanitized focal length/focus/blade data; and `post_process.wgsl` uses those values for a rotated disk/bokeh kernel inside the current final-pass approximation. When DoF is active, the neutral effect-stack node now also produces `postprocess.depth-of-field.coc` and `postprocess.depth-of-field.bokeh`; the compiled-scene renderer imports those names from concrete `OffscreenTarget` scratch textures so executor/resource validation can see the intended split-pass contract before dedicated CoC/bokeh writer passes exist. Near/far separation, temporal CoC history, a half-resolution bokeh policy, and split DoF graph passes remain future renderer work rather than current submit stats.

History validity is also visible at the diagnostics boundary. `RenderStats.last_frame_history_status` remains the structured source of current/previous handles, previous-frame usability, invalidation reason, presentation target size, and internal render size, while `collect_runtime_diagnostics(...)` mirrors numeric state under `render.history.*`: handle presence, `previous_available`, aggregate `invalidated`, `target_width` / `target_height`, `render_width` / `render_height`, and one-hot invalidation reason rows. Target-size changes still report `viewport_resized`; internal render-size changes, including dynamic-resolution scale changes with the same viewport size, report `render_size_changed`. This gives tooling a stable ghosting/resize/input-change signal without reading renderer history textures.

Advanced-slot readiness now has the same submit-level diagnostics bridge. `collect_runtime_diagnostics(...)` mirrors AA requested/effective/history/fallback state as `render.anti_alias.*`, GPU particle feedback as `render.particle.gpu.*`, VG budgets/visibility/page residency/debug/cull/readback-source counters as `render.virtual_geometry.*`, HGI probe/cache/scene/surface-cache/voxel payload counters as `render.hybrid_gi.*`, provider availability/report/degradation state as `render.advanced_provider.*`, and Solari requested/status/experimental/degradation state as `render.solari.*`. These rows are neutral `RenderStats` projections only; runtime tooling can inspect advanced readiness without depending on plugin-private VG/HGI/Solari state.

Material readiness now follows the same bridge. `RenderStats.last_material_*` carries material count, ready count, fallback count, validation-error count, and non-blocking diagnostic count after `ResourceStreamer::ensure_scene_resources(...)` folds material reports into submit stats. `collect_runtime_diagnostics(...)` mirrors those rows under `render.material.*`, so tools can show material readiness and authoring/import diagnostics without scanning every material report.

Light readiness is exposed by family. `RenderStats.last_*_light_*` carries total, ready, and degraded counts for directional, point, spot, ambient, and rect lights after `RenderLightReadinessReport` applies the current renderer support rules. `DiagnosticStore` mirrors them under `render.light.<family>.count`, `ready_count`, and `degraded_count`, so tools can see which extracted light families are present but intentionally degraded.

Mesh queue preparation now computes counters for early-z eligible draws, prepared versus dynamic geometry, indirect draws, and static/dynamic/GPU-instancing candidate groups. Those counters live under `zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs`, are consumed by the compiled-scene path to keep transparent draws out of the depth prepass, and are carried through `SceneRendererCompiledSceneOutputs` into `RenderStats.last_mesh_*` after a successful submit. `collect_runtime_diagnostics(...)` mirrors them into `DiagnosticStore` under `render.mesh.queue.*` paths so tools can observe current-frame queue readiness without reading renderer-private draw lists. They are queue-preparation diagnostics, not generic render-asset or mesh allocator diagnostics.

Sprite readiness and queue preparation follow the same public diagnostics rule. `RenderStats.last_sprite_*` carries sprite count, ready count, texture fallback count, graph executed pass count, adjacent draw-batch count, batched sprite count, generated vertex count, and per-phase batch counts. `DiagnosticStore` mirrors these as `render.sprite.*` and `render.sprite.queue.*` count rows, keeping Core2d batching and fallback visibility available to runtime/editor tooling without promoting Bevy-style sprite binning or per-view pipeline specialization.

Runtime UI composition stats are also product diagnostics. `RenderStats.last_ui_*` carries UI command, quad, text payload, image payload, clipped command, and UI graph executed pass counts. `DiagnosticStore` mirrors these as `render.ui.*`, giving runtime/editor tooling a stable way to see screen-space UI payload and graph placement scale without reading the retained UI graph or WGPU UI pass internals.

## Bevy Gap Classification

| Bevy diagnostics area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Render sub-app diagnostics lifecycle | Zircon has a submit-time `RenderStats` snapshot and `query_stats(...)`; it does not have a separate render app, render graph begin/resolve/finish diagnostics systems, or main/render-world diagnostic mutex. | Add a diagnostics bridge that keeps render submit stats and any future GPU timing data consumable from runtime `DiagnosticStore` without exposing renderer-private state. |
| CPU/GPU pass timing | RenderDoc markers and graph execution records identify passes, but no CPU elapsed-time, GPU timestamp-query, or pipeline-statistics measurements are recorded per pass. | Add per-pass timing/span recording with backend capability fallback: GPU stats on supported backends, CPU-only on unsupported ones. |
| Pipeline statistics | `RenderStats` reports graph counts and runtime product counters, not shader invocations, primitive counts, or pipeline-statistics query results. | Add a backend-gated pipeline-statistics recorder before claiming Bevy `RenderDiagnosticsPlugin` parity. |
| Render asset diagnostics | Material, sprite, texture fallback, mesh queue-prep, VG, and HGI counters are product-specific stats, but there is still no generic `RenderAsset` diagnostic plugin family. | Add typed render-asset residency/count diagnostics and erased-asset count diagnostics if render asset storage becomes generic enough to support it. |
| Mesh allocator diagnostics | Current docs track mesh/material readiness, graph execution, and public mesh queue candidates; there is no Bevy-style mesh allocator slab/byte/allocation diagnostic. | Add allocator-level mesh memory diagnostics once Zircon's mesh allocator has stable slab/residency ownership; keep queue-prep counters separate from allocator residency. |
| Pipelined rendering visibility | Zircon's runtime framework submit path is synchronous from the caller's perspective; no Bevy-like render thread/sub-app overlap diagnostics exist. | Keep this as a future scheduling milestone; do not conflate current submit stats with pipelined rendering telemetry. |

## M10U Render Diagnostics And Profiling Bridge Gate

M10U uses this document as the render-product side of the M10.8 gate. The current product state is useful, but narrower than Bevy's `RenderDiagnosticsPlugin`: `RenderStats` is a frame submit snapshot, not a render graph diagnostics recorder with begin/resolve/finish lifecycle, GPU timestamp query buffers, pipeline statistics, or generic render-asset diagnostics.

The runtime diagnostics bridge is intentionally small today. `zircon_runtime/src/core/diagnostics/render.rs` wraps the last queried `RenderStats` in `RuntimeRenderDiagnostics`, and `collect_runtime_diagnostics(...)` copies frame/viewport counts, capability gates, history validity, planned/executed graph counters, M5/M6 material/light/mesh/sprite/UI readiness, product postprocess graph scale, and the M7 effect-stack readiness summary into the runtime-owned `DiagnosticStore`. Capability rows use `render.capability.*`; history rows use `render.history.*`; graph rows use the legacy `render.last_graph_executed_pass_count` plus `render.graph.*`, including sparse reservation lifetime, sparse reservation slot counts, and feature execution rows such as `render.graph.executed_shadow_pass_count`; postprocess graph rows use `render.post_process.graph.*`; effect-stack rows use stable count/bool paths: `render.post_process.effect_stack.enabled`, `render.post_process.effect_stack.active_family_count`, `render.post_process.effect_stack.approximated_family_count`, and `render.post_process.effect_stack.missing_resource_count`. LUT renderer readiness rows use `render.post_process.lut.request_count`, `ready_count`, `fallback_count`, `texture_2d_strip_ready_count`, `texture_3d_request_count`, and `unsupported_shape_count`, distinguishing an enabled LUT family that reached an authored texture from one rendered with the fallback LUT and exposing whether the authored texture matches the current 2D binding contract. Material rows use `render.material.*`, light rows use `render.light.<family>.*`, mesh queue rows use `render.mesh.queue.*`, sprite rows use `render.sprite.*` and `render.sprite.queue.*`, and UI rows use `render.ui.*`. That is enough for tools to see current product readiness, but not enough to claim Bevy-style render diagnostics parity.

The profiling feature is adjacent evidence, not a substitute. It can record CPU spans for submit, present, capture, lock waits, graph stages, and graph passes, then export native/perfetto/hotspot artifacts. It does not record GPU timestamp queries, shader invocation/primitive pipeline statistics, render-asset residency counts, mesh allocator slab bytes, or render-thread overlap telemetry. RenderDoc markers stay in the debugging lane; Bevy's profiling docs explicitly separate RenderDoc from GPU profilers.

| M10.8 evidence family | Current Zircon state | Promotion requirement |
| --- | --- | --- |
| Product readiness | `RenderStats` records product counters, fallback/readiness reports, graph counts, and advanced status. | Preserve these counters as product diagnostics and keep them visible through `RuntimeDiagnosticsSnapshot`. |
| DiagnosticStore bridge | `collect_runtime_diagnostics(...)` records frame, viewport, capability, history, planned/executed graph, postprocess graph, M5/M6 material/light/mesh/sprite/UI, M7 effect-stack, and LUT request/ready/fallback/shape count paths into `DiagnosticStore`. | Add stable paths for pass timing, pipeline/cache status, present/capture failures, and resource residency before overlay/log consumers treat render diagnostics as complete. |
| CPU pass timing | profiling spans cover submit/present/capture and graph stage/pass CPU work in profiling builds. | Make pass-level timing visible as diagnostics or clearly link profiling artifacts to the diagnostics snapshot during promotion. |
| GPU timing and pipeline statistics | no timestamp-query recorder or pipeline-stat query rows exist. | Gate GPU rows by backend capability, report unavailable states, and keep CPU-only fallback explicit. |
| Render assets and mesh allocator | material/sprite/light readiness stats exist; no generic render-asset or mesh allocator diagnostic family exists. | Add generic render-asset residency/counts and mesh allocator byte/allocation diagnostics only after storage ownership is stable. |
| Pipelined telemetry | submit remains synchronous. | Future render-thread handoff, overlap timing, and shutdown telemetry must be separate from synchronous submit stats. |

Promotion requires focused render diagnostics snapshot tests, `DiagnosticStore` log schedule tests, profiling artifact smoke, profiling-build render graph span tests, and `cargo check -p zircon_runtime --lib --locked` in a quiet build window.

2026-05-26 M10W validation evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_diagnostics --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked diagnostic_store --jobs 1 --message-format short --color never`: PASS, 5 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --lib profiling --profile profiling --features profiling --locked --jobs 1 --message-format short --color never`: PASS, 20 matching profiling tests passed after an initial cold profiling-profile compile timed out before test execution.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --profile profiling --features profiling --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.

This promotes the current `RuntimeDiagnosticsSnapshot` / `DiagnosticStore` bridge and CPU profiling artifacts only. It does not claim GPU timestamp queries, pipeline-statistics rows, generic render-asset diagnostics, mesh allocator diagnostics, or pipelined render-thread telemetry.

2026-06-03 render-main-chain SSR normal/depth-backend follow-up evidence: WSL `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed with existing warnings. Windows focused tests passed for the GL fallback shader, viewport-depth fallback source generation, graph-bound SSR normal status, effect-stack resource report, effect-stack normal/depth graph declaration, and the extended product-node regression. WSL GPU bridge execution remains environment-blocked: a warning-enabled lib-test compile hit a Rust 1.94.1 diagnostic ICE while emitting existing warnings, the warning-suppressed GL run of `render_framework_stats_report_effect_stack_product_node_when_authored` ended in a driver-level SIGSEGV during thread TLS teardown, and `WGPU_BACKEND=vulkan` returned `NoAdapter`.

## Bevy Presentation Surface Evidence

Bevy keeps camera targets and window presentation explicit instead of treating all rendered output as one swapchain path. `dev/bevy/crates/bevy_camera/src/camera.rs:22-58` defines `Viewport` as a physical rectangle inside a render target and clamps it to the target size. `camera.rs:814-855` defines `RenderTarget::{Window, Image, TextureView, None}` and normalizes them to concrete target keys; the `None { size }` target represents a camera with no color target, useful for prepass-only rendering. `dev/bevy/crates/bevy_render/src/camera.rs:263-300` resolves the normalized target into `RenderTargetInfo` from a window, image, manual texture view, or explicit no-color size, and reports missing window/image/texture-view targets as structured errors at `camera.rs:322-331`.

Window surfaces are render-app resources in Bevy. `dev/bevy/crates/bevy_render/src/view/window/mod.rs:31-45` registers `ExtractedWindows`, `WindowSurfaces`, `extract_windows`, `create_surfaces`, and `prepare_windows`. `mod.rs:49-99` stores the extracted window size, present mode, swapchain texture view, and `SurfaceTexture`, then presents by taking the current surface texture and calling `present()`. `mod.rs:358-458` creates/configures raw-handle WGPU surfaces, chooses a surface format, updates size and present mode, and reconfigures the surface on resize or present-mode changes. `mod.rs:465-508` implements present-mode fallback, ending in FIFO-compatible choices when a requested mode is not advertised.

Bevy screenshots are also target-aware. `dev/bevy/crates/bevy_render/src/view/window/screenshot.rs:49-111` models screenshot requests as a `Screenshot(RenderTarget)` component and returns a `ScreenshotCaptured` image asynchronously. `screenshot.rs:406-439` wires the screenshot plugin into the main app and render app, including `extract_screenshots` and `prepare_screenshots` before view targets are prepared. `screenshot.rs:596-603` copies the prepared screenshot texture into a readback buffer, and `screenshot.rs:647-682` maps the buffer asynchronously, strips row padding, and sends a CPU `Image` result.

## Zircon Presentation State

Zircon's camera target vocabulary is intentionally narrower today. `zircon_runtime/src/core/framework/render/camera.rs:84-94` defines `RenderCameraTarget::{PrimarySurface, Texture, Headless}` with `PrimarySurface` as the default. `submit_frame_extract/build_frame_submission_context/target_resolution.rs:4-38` resolves `PrimarySurface` to the viewport record size, resolves `Headless { size }` to a clamped offscreen size, rejects `Texture(_)` with `UnsupportedCapability("camera texture render target")`, and rejects `Headless` on the surface-present path with `UnsupportedCapability("headless camera surface present")`.

The runtime framework separates offscreen submit/capture from native surface present. `present_frame_extract.rs:24-104` builds the normal submission context, rejects non-primary camera targets for surface present, requires a bound viewport surface, prepares runtime submission, builds the runtime frame, leases the bound surface, and calls `renderer.present_frame_with_pipeline(...)`. The backend surface path in `render_backend/viewport_surface.rs:55-80` acquires the current WGPU surface texture, blits the rendered offscreen source into it, calls `surface_texture.present()`, and treats outdated/lost/timeout/occluded surfaces as nonfatal or reconfigurable. `viewport_surface.rs:270-330` configures the surface with clamped size, an SRGB-preferred format, `AutoVsync` or FIFO-biased present mode, and fixed frame latency.

The focused surface-target tests document this product boundary. `zircon_runtime/src/graphics/tests/surface_targets.rs:111-152` proves offscreen submit/capture survives surface unbind and that `Headless { size }` controls captured frame size. `surface_targets.rs:155-193` proves texture targets and headless surface-present requests return explicit unsupported-capability errors instead of silently falling back to primary output. `surface_targets.rs:196-206` guards the surface present implementation against regressing into readback-based present fallback.

## Presentation Gap Classification

| Bevy presentation area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Window surface lifecycle | Zircon has viewport records, raw Win32 surface binding, WGPU surface configuration, present blit, and explicit missing-surface errors. It does not yet have Bevy's render-app `ExtractedWindows` / `WindowSurfaces` resource lifecycle or broad platform surface owner model inside the render plan. | Keep native-window ownership in `zircon_app`/platform, but expose enough render-side diagnostics for bound/unbound/resized/present-mode state without mixing platform input work into the renderer. |
| Image and texture render targets | Bevy supports `RenderTarget::Image` and manual `TextureView`; Zircon has a neutral `Texture(handle)` target but currently rejects it before submit. | Land GPU texture residency/writeback and render-to-texture scheduling before claiming Bevy image/texture-view target parity. |
| No-color / headless target semantics | Bevy `RenderTarget::None { size }` is no-color, useful for prepass-only rendering. Zircon `Headless { size }` renders to an offscreen color target that can be captured. | Record the intentional divergence. Add a true no-color/depth-only target only if depth-prepass or shadow-only camera workflows need it. |
| Screenshot and capture workflow | Bevy has an async screenshot component, per-target preparation, GPU copy-to-buffer, row-padding cleanup, and image callback. Zircon exposes viewport `capture_frame(...)` and RenderDoc capture hooks, but no Bevy-like screenshot request/result pipeline. | Add a screenshot/capture request API that can target primary, headless, and future texture targets, return structured async results, and integrate with dev/CI artifact paths. |
| Present-mode and surface diagnostics | Zircon chooses an advertised present mode internally and records present failures as framework errors; Bevy exposes present-mode fallback decisions in the window render path. | Add surface-format/present-mode/fallback diagnostics to `RenderStats` or `DiagnosticStore` once the platform/window session's surface lifecycle stabilizes. |

## M10R Default 2D And Presentation Base Gate

M10R ties the default 2D and presentation milestones together only at the acceptance boundary. Bevy keeps the same separation: `2d_bevy_render` is the renderer collection for 2D apps, while camera targets and screenshots are target-aware render output contracts. A sprite draw can prove renderer product state only when Core2d extraction, queueing, graph pass execution, resource fallback, and stats are visible; a screenshot or headless capture can prove presentation only when the target and readback path are explicit.

The Zircon side follows that split. [Render Sprite Contracts](../core/framework/render/sprite.md) owns the default Core2d sprite base: `Sprite2dComponent` projects into `RenderSpriteSnapshot`, `SpriteExtract` stays separate from particles, `default_core2d()` exposes sprite graph passes, and sprite stats report ready/fallback counts. This document owns the presentation base: `PrimarySurface` uses the bound viewport surface, `Headless { size }` uses offscreen capture size, `Texture(handle)` is still explicit unsupported, and headless surface-present is rejected before it can fall back to the primary surface.

The promotion evidence must include both sides. `render_product_sprite` and the default Core2d pipeline compile test cover the current 2D base; `camera_target` and `surface_targets` cover target routing, offscreen capture size, missing target errors, and swapchain present rather than readback fallback. Mesh2d/SpriteMesh, slice/tile sprites, binned batching, per-view 2D pipeline specialization, target-aware async screenshot requests, render-to-texture, and manual texture views remain open after M10R.

## Bevy PBR Material And Lighting Evidence

Bevy's PBR baseline is a product family, not just a shader. `dev/bevy/crates/bevy_pbr/src/lib.rs:130-156` defines `PbrPlugin` with prepass, deferred lighting, GPU instance buffer building, and glTF StandardMaterial defaults. The plugin loads the PBR shader library set at `lib.rs:179-198`, registers `StandardMaterial` and `MaterialPlugin::<StandardMaterial>` at `lib.rs:203-216`, adds SSAO, fog, lightmap, light probes, volumetric fog, SSR, transmission, clustered decals, and contact shadows at `lib.rs:217-230`, syncs directional/point/spot/rect/ambient lights at `lib.rs:232-239`, adds atmosphere and GPU clustering at `lib.rs:240-244`, and conditionally adds deferred PBR lighting at `lib.rs:251-252`.

Bevy's `StandardMaterial` carries a broad authored and GPU-facing contract. `dev/bevy/crates/bevy_pbr/src/pbr_material.rs:26-57` starts the material with base color, UV channel, and texture dependency bindings, while later fields cover emissive, metallic/roughness, normal/occlusion, alpha, transmission, clearcoat, anisotropy, and parallax families. The shader-visible flags at `pbr_material.rs:967-1003` distinguish texture slots, double-sided, unlit, normal-map options, fog, parallax, transmission, clearcoat, anisotropy, specular, and alpha modes. `pbr_material.rs:1010-1056` packs the GPU uniform with base color, emissive, roughness, metallic, transmission, thickness, IOR, clearcoat, anisotropy, flags, alpha cutoff, and parallax parameters.

The Bevy material pipeline is asset-driven and render-phase aware. `dev/bevy/crates/bevy_pbr/src/material.rs:74-144` defines `Material` as an `Asset + AsBindGroup` abstraction used with `Mesh3d` and `MeshMaterial3d`; `material.rs:289-342` initializes specialized material pipeline caches, material instances, bind group allocators, draw commands for shadow/transparent/opaque/alpha-mask phases, material mesh specialization, queueing, bind group preparation, shadow specialization, and shadow queueing. `mesh_material.rs:8-41` makes the mesh-to-material handle component explicit, while `material_bind_groups.rs:36-115` shows the bindless/non-bindless allocator and slab resource tracking surface that backs material bind groups.

The shader side confirms why PBR parity must include both forward and deferred lighting. `dev/bevy/crates/bevy_pbr/src/render/pbr.wgsl:65-89` creates `PbrInput` from StandardMaterial bindings, handles alpha discard, writes deferred/prepass output when configured, or applies PBR lighting plus in-shader post-lighting in forward mode. `deferred/deferred_lighting.wgsl:59-86` reconstructs `PbrInput` from the deferred G-buffer, folds in SSAO when available, and applies the same PBR lighting path. `render/pbr_lighting.wgsl:34-122` defines the BRDF and lighting input model for point, spot, and directional lights, including clearcoat and anisotropy variants. Cluster data is its own GPU surface: `cluster/cluster.wgsl:4-24` defines point/spot/probe/decal clusterable object kinds and cluster metadata, and `render/light.rs:1316-1343` writes point/spot lights into `GpuClusteredLight`; `render/light.rs:1519-1624` builds per-view `GpuLights` with ambient, directional, clusters, and rect-light storage.

## Zircon PBR Material And Lighting State

Zircon's current baseline intentionally lands below Bevy's full PBR plugin breadth. The neutral material descriptor in `zircon_runtime/src/core/framework/render/material/standard_material.rs:8-23` carries name, dependency set, base color, base-color/normal/metallic-roughness/occlusion/emissive textures, metallic, roughness, emissive, alpha mode, unlit, double-sided, and fallback policy. `render/material/readiness_report.rs:31-58` records validation errors and fallback usage so runtime stats can distinguish usable material rows from fallback-dependent rows.

Concrete GPU preparation currently projects that descriptor into a smaller runtime material. `resource_streamer_ensure_material.rs:18-84` loads the material, optional shader contract, and readiness report; `resource_streamer_ensure_material.rs:89-119` resolves standard texture slots; `resource_streamer_ensure_material.rs:150-195` merges shader readiness, handles blocking validation, and constructs `MaterialRuntime`. The material report now includes asset-owned shader payload readiness rows from `ShaderAsset::readiness_report()`, so invalid shader entry stages and duplicate or empty shader definitions are visible beside schema, WGSL capture, texture fallback, and missing runtime WGSL diagnostics. `material_runtime.rs:27-42` stores the runtime scalar/texture fields, `PipelineKey`, and readiness report; `pipeline_key.rs:4-17` keys shader revision, double-sided, alpha blend/mask/cutoff, unlit, and standard texture-slot presence.

Renderer texture preparation now accepts packed RGBA8 D2 array payloads and complete RGBA8 mip chains through `TextureAsset::upload_readiness(...)`. `GpuTextureResource::from_asset(...)` uploads each mip/layer rectangle into the backing WGPU D2 texture in mip-major, layer-major order, matching the neutral RHI `TextureCopyRegion` direction. The current material and sprite bind group ABI still expects a D2 sampled texture, so the prepared resource creates a first-layer D2 view for existing PBR/sprite texture slots. This lets six-layer skybox/cubemap-style and sprite-array payloads reach GPU residency without silently changing shader sampling semantics; array or cube sampling needs an explicit shader/material slot contract and bind-group layout before it can be claimed.

Zircon has real renderer hooks, but they are still narrower than Bevy's PBR path. `deferred_scene_resources/record_gbuffer_geometry.rs` records the deferred geometry pass by binding scene/model/texture/geometry data for each mesh draw, and that pass is now dispatched through the `deferred.gbuffer` graph executor. The deferred graph now declares a backed `gbuffer-material` target beside `gbuffer-albedo` and `gbuffer-normal`: `OffscreenTarget` owns an `Rgba8Unorm` material texture, `import_frame_targets(...)` imports it into graph execution resources, `gbuffer-mesh` writes it, and `deferred-lighting` reads it. The current shader contract stores material uniform channels as metallic, roughness, and occlusion inputs for the fullscreen lighting approximation, and final postprocess also consumes the graph-written material roughness channel to attenuate the SSR seed; this is a renderer-owned material-channel G-buffer, not a full Bevy `StandardMaterial` GPU ABI or bindless material allocator. `deferred_scene_resources/execute_lighting.rs` therefore runs a fullscreen deferred lighting pass over albedo, normal-prepass, material, background, and scene-color targets through the `lighting.deferred` graph executor. `execute_clustered_lighting.rs:14-87` writes directional-light data into a fixed clustered-light buffer and dispatches a compute culling pass, but it currently operates on `RenderDirectionalLightSnapshot` only. The shadow feature now contributes a live, side-effectful `shadow-map` graph pass with `shadow.map` executor metadata and a transient `Depth32Float` sampled/render-attachment texture; the concrete executor materializes that graph resource and records a depth-only WGPU render pass with the graph clear/store attachment ops. `render_framework_stats_report_shadow_map_graph_execution` proves the submitted WGPU RenderFramework path reports the `shadow-map` pass, `shadow.map` executor id, and graph debug marker instead of limiting the evidence to compile-time graph shape. It intentionally does not draw shadow casters, bind a compare sampler, filter cascades, or feed lighting yet. Bevy-style point/spot clustered light shading, rect area-light shading, shadow sampling, contact shadows, probes, transmission, and advanced material lobes remain outside the accepted baseline.

## PBR Material And Lighting Gap Classification

| Bevy PBR area | Zircon product state | Completion requirement |
| --- | --- | --- |
| StandardMaterial surface | Zircon covers the core base-color, normal, metallic/roughness, occlusion, emissive, alpha, unlit, and double-sided descriptor surface. | Add the missing Bevy StandardMaterial families deliberately: reflectance/specular, transmission/thickness/IOR/attenuation, clearcoat, anisotropy, parallax/depth maps, UV transforms/channels, lightmap interaction, and debug/shader-def controls. |
| Material bind groups | Zircon prepares runtime textures and pipeline keys, but material binding is not Bevy's `AsBindGroup`/bindless allocator model. | Land explicit bind-group layout reflection, slot validation, fallback resource residency, material cache invalidation, and bindless/non-bindless policy before claiming Bevy-like material plugin parity. |
| Phase-specialized material pipeline | Zircon has Core3d phases plus forward/deferred pipeline assets and alpha-derived queues. | Add material-specialized pipeline cache states, per-material shader defs, shadow/deferred/prepass variants, OIT/transmission phases, and structured pipeline-error diagnostics. |
| Physically based lighting | Zircon consumes authored ambient light and one basic directional slot, has a limited directional clustered-light compute path, and records a graph-owned depth-only `shadow-map` placeholder pass. | Implement real shadow caster draw lists, compare-sampler/material variants, shadow sampling in lighting, point/spot clustered lighting, rect/area lights, contact shadows, lightmaps, probes/IBL, SSAO/SSR coupling, clearcoat/anisotropy/transmission lighting, and per-view light visibility before marking full PBR lighting complete. |
| Deferred parity | Zircon records a G-buffer geometry pass, backs albedo/normal/material G-buffer resources, and runs a fullscreen lighting pass that consumes material channels. | Align the G-buffer contract with authored standard-material flags, stable scalar uniform slots, normal/motion/depth prepasses, deferred lighting pass IDs, SSAO/specular occlusion, and fallback handling for unlit and unsupported material modes. |
| Authoring and assets | Zircon has runtime descriptors and asset-side material files, but this docs slice does not enter `.zmaterial` or material editor implementation. | Sequence `.zmaterial`, material editor projection, shader-contract authoring, and asset hot-reload with the active asset/material lane rather than folding it into render submit docs. |

## M10S Default 3D PBR And Light Gate

M10S uses this document as the default 3D/PBR/light side of the M10.5 gate. The current Zircon state is a foundation, not parity: material descriptors and readiness reports are typed, runtime materials carry a compact `PipelineKey`, ambient light can feed the shared scene uniform, light readiness splits ready/degraded families, the renderer records a G-buffer pass, fullscreen deferred lighting pass, and a limited directional clustered-light compute pass.

The gate remains explicit about what is missing. Bevy's PBR product includes StandardMaterial lobes for transmission, clearcoat, anisotropy, parallax, specular/reflectance, UV channel variants, material bind-group generation, phase specialization, point/spot clusterable lights, rect lights, shadows, contact shadows, probes, lightmaps, SSAO, SSR, fog, volumetrics, and deferred/forward coupling. Zircon must preserve those as open gaps instead of letting an ambient uniform, one directional slot, a G-buffer skeleton, a depth-only shadow-map placeholder, or advanced GI provider close the default 3D milestone.

Promotion evidence must be family-based. Material tests need to distinguish covered descriptor fields from missing Bevy families; light tests need ready/degraded evidence for ambient, directional, point, spot, rect, probe, and baked light families; submit tests need material fallback and light stats; renderer tests need forward/deferred phase and pipeline coverage. `.zmaterial`, material editor projection, shader import resolution, and asset hot-reload remain owned by the active asset/material lane.

The basic forward and deferred mesh shaders share `SceneUniform`. When preview lighting is enabled, `SceneUniform::from_frame(...)` now reads active authored ambient lights from `RenderFrameExtract::lighting.ambient_lights`, accumulates `color * intensity`, and writes that value to `ambient_color`. If no ambient light is authored, the renderer keeps the existing preview fallback ambient value. This closes the first concrete ambient-light consumption step without changing `.zshader` / `.zmaterial` ownership or adding a new material pipeline.

Render submit stats now split light slots by renderer readiness as well as by total count. `RenderLightReadinessReport` in `render::light` owns the rule: `last_ambient_light_ready_count` / `last_ambient_light_degraded_count` report whether authored ambient slots are usable by the current `SceneUniform` path; `last_directional_light_ready_count` is capped to the single directional slot currently consumed by the basic `SceneUniform` path; point/spot ready counts remain zero because those lights are extracted but not shaded by the current default PBR path; rect lights remain degraded until the PBR/area-light shader path lands. This mirrors the Bevy distinction between ambient/direct/clusterable/rect light GPU representations without letting the advanced lighting gap hide inside a single total count.

Core pipeline selection is neutral framework data. Cameras select `CorePipelineKind::Core2d` for orthographic projections and `CorePipelineKind::Core3d` for perspective projections. Unset viewport submit uses that extract-owned pipeline kind to choose the built-in Core2d or Forward+ Core3d pipeline, while explicit viewport pipelines and quality-profile overrides remain authoritative and are rejected at compile time if their `core_pipeline` does not match the submitted extract.

Dynamic resolution is also neutral framework data, not a backend upscaler. `ViewportCameraSnapshot` carries `RenderDynamicResolutionSettings`; `RenderViewExtract::effective_view_size()` remains the physical viewport, present, and UI damage size, while `RenderViewExtract::effective_render_size()` applies the clamped render scale for internal scene resources only. `RenderPipelineAsset::compile(...)` uses that render size for transient texture and buffer descriptors such as `scene-color` and `scene-depth`, and keeps external `viewport-output` imported as the unscaled presentation target. Submit status preserves the same split: `FrameSubmissionContext::size()` is the imported/presentation target, `FrameSubmissionContext::render_size()` is the internal graph extent, `RenderStats.last_frame_target_size` / `last_frame_render_size` expose both values, and `FrameHistoryStatus.target_size` / `render_size` records the dimensions used for temporal validity. Future FSR/TAA/upscale passes therefore have a stable size contract without making current UI or swapchain ownership depend on scaled render extents; this does not claim a current upscale pass or dynamic quality controller exists.

`HistoryResolve` is no longer part of the default effective pipeline. `RenderFeatureQualitySettings::default()` leaves `history_resolve` disabled, `BuiltinRenderFeature::HistoryResolve` requires explicit opt-in, and profile compilation enables it only when a profile calls `with_history_resolve(true)`. This keeps default Core3d rendering free of scene-color temporal blending until motion vectors, reprojection, camera-cut detection, and disocclusion checks exist.

`GeometryExtract` carries phase queues derived from material alpha mode plus the selected pipeline. Production world extraction reads the alpha hint stored on each `MeshRenderer` and creates phase inputs from the sorted mesh rows, so mesh draw construction can consume aligned opaque, alpha-mask, and transparent queues instead of falling back to raw mesh-vector order.

Pipeline compile validates that declared renderer stages with product phases have matching `RenderPipelineAsset.phase_mapping` entries. The enforced stage-to-phase mapping covers 2D mesh stages, 3D mesh stages, depth prepass, shadow, deferred, postprocess, UI, overlay, and debug; lighting and ambient occlusion remain product-phase-neutral until a dedicated phase exists. Runtime graph execution calls declared graph stages through `execute_graph_stage`; UI, overlay, sprite, preview sky, depth/normal prepass, shadow-map, forward mesh, Deferred G-buffer, Deferred lighting, Deferred transparent mesh execution, SSAO, clustered-light culling, bloom extraction, and final `post.stack` composition are graph-owned. `RenderPassExecutorRegistry` now owns only executor-id registration, explicit plugin/no-op policy, execution lookup, and compiled-pipeline validation; concrete built-in behavior lives in `builtin_postprocess_executors.rs`, `builtin_scene_executors.rs`, and the existing `preview_sky_executor.rs`, so future renderer-specific executor work has a semantic owner instead of extending one registry file. Descriptor-provided plugin executor ids are no longer backfilled with runtime no-op executors: linked plugin descriptors declare graph topology, and explicit `RenderPassExecutorRegistration` rows prove which plugin owns each pass body. Post-process graph node accounting is now a `RenderGraphStageExecution::record_post_process_graph(...)` responsibility: it clones the effective frame graph, runs node availability against graph-bound resources, and writes both the graph and executed node names into the same `RenderGraphExecutionRecord` without letting `render_compiled_scene(...)` reach into the record/resource pair directly.

M8 neural-network support is represented as a compute-backed capability slot, not as an implemented neural renderer. `RenderCapabilityKind::NeuralCompute`, `RenderFeatureCapabilityRequirement::NeuralCompute`, and `RenderBackendCaps.supports_neural_compute` give plugin render features a stable gate for future ML denoisers, upscalers, or inference-backed passes. `BuiltinRenderFeature::NeuralCompute` and its `neural_compute` descriptor are descriptor-only: explicit feature opt-in records the backend capability requirement but adds no graph pass or executor. Plugin neural descriptors remain the owner for executable async-compute passes, and `RenderPipelineCompileOptions` treats `NeuralCompute` as an explicit capability gate so those plugin passes stay out of the graph until capability opt-in. Runtime activation now checks the full chain: descriptor-only neural passes fail `register_pipeline_asset(...)` when `plugin.neural.*` has no explicit `RenderPassExecutorRegistration`, and executor-backed neural passes still fail `set_pipeline_asset(...)` when the active backend capability summary lacks `neural_compute`. The WGPU backend reports the capability as unsupported by default, capability summaries and diagnostics expose that truth, and compiled pipeline validation rejects a descriptor that requires `neural_compute` until a real backend/provider path opts in. Runtime still does not own model weights, tensor lifetimes, or plugin-private neural state.

Post-process effect-stack settings now travel through the same submit path as bloom and color grading. `RenderPostProcessEffectStackSettings` carries neutral tonemap, LUT, blur, depth-of-field, screen-space-reflection, vignette, film-grain, dither, chromatic-aberration, and fog parameters on `PostProcessExtract`; `build_frame_submission_context(...)` preserves the effective settings and derives an enabled-only `effect-stack` product node before final composite. The compiled-scene resource registry imports `postprocess.effect-stacked` as a graph alias for stats/resource validation. `build_post_process_params(...)` now encodes those resolved settings plus camera depth/projection/view-basis parameters into the WGPU post-process uniform block, and `post_process.wgsl` consumes them for baseline tonemap/exposure, authored-or-fallback LUT texture sampling, blur, camera-linearized scene-depth DoF-radius sampling, projection-aware bounded SSR with camera-view-space normal direction, material roughness, scene-depth/thickness testing, distance fade, and screen-edge fade, vignette, grain, dither, chromatic aberration, and view-depth-weighted fog. The WGPU postprocess bind group treats positive LUT intensity as the request signal, selects an authored 2D or 2D-strip LUT texture through `ResourceStreamer` at binding 10 when prepared, selects a decoded 3D LUT through the dedicated post-process LUT cache at binding 12 when the retained descriptor is `Texture3d`, binds a clamp/linear LUT sampler at binding 13 for the 3D path, falls back to the renderer-owned 64x1 S-curve LUT or 2x2x2 identity cube when the selected authored texture is unavailable, binds graph `scene-depth` at binding 11, binds graph `gbuffer-normal` at binding 14 for SSR direction, and binds graph `gbuffer-material` at binding 16 only when an active pass writes it. `RenderColorLookupTextureLayout` records whether authored LUT content is `Auto`, `Texture2dStrip { size }`, or `Texture3d { size }`; `ResourceStreamer` keeps each prepared texture's neutral `RenderImageDescriptor` so array and malformed shapes cannot be accidentally bound into the wrong WGPU slot. Text `.cube` LUT files now enter through the built-in `zircon.builtin.texture.cube_lut` importer as linear D3 RGBA8 `TextureAsset` payloads and explicitly reject 1D shaper sections until shaper-aware baking is designed, so submit-time authored LUT handles can drive the same renderer-private 3D path without `app`, `editor`, or framework code depending on WGPU. `PostProcessParams.effect_flags.y` records the actual shader binding mode selected for the frame, and the 3D branch samples normalized texel-center coordinates with `textureSampleLevel(...)` so endpoint colors remain stable while intermediate values interpolate between LUT cells. `PostProcessParams.effect_projection` carries perspective focal scales and orthographic half extents so the final-pass SSR path can reconstruct a view-space origin from depth and project ray candidates back to screen pixels before depth/thickness acceptance, while `effect_view_x/y/z` carries the camera basis used to transform the world-space normal buffer into the view space used by that ray march. `RenderStats.last_post_process_effect_stack_report` records `effect-stack.lut.texture` only when LUT intensity is authored without a texture handle, records `effect-stack.lut.texture-layout` for invalid explicit layout sizes, treats a supplied authored LUT as renderer-bound rather than approximation-backed, and clears `effect-stack.ssr.normal` when the graph declares `gbuffer-normal`; material roughness stays a renderer-bound G-buffer input with black-fallback roughness 1.0 when no writer exists. `RenderStats.last_post_process_lut_{request,ready,fallback}_count` plus `last_post_process_lut_2d_strip_ready_count`, `last_post_process_lut_3d_request_count`, `last_post_process_lut_unsupported_shape_count`, and `render.post_process.lut.*` diagnostics expose whether the enabled LUT family requested a texture, prepared it, used fallback, matched the 2D-strip contract, used the renderer-private 3D path, or supplied an unsupported shape. Advanced LUT authoring policies, focus/lens DoF resources, temporal SSR resolve, specular occlusion, depth/reflection pyramids, and dedicated split passes keep explicit regression labels instead of being hidden behind one enabled product node.

The submit path also resolves a neutral post-process volume stack before the product graph is built. `RenderPostProcessVolumeStack` carries global/local volumes with priority, weight, render-layer mask, local blend influence, and optional bloom/color-grading/effect-stack profile overrides. `PostProcessExtract::resolved_settings_for_layers(...)` filters that stack by the current camera render layers, blends it into effective per-camera settings, and then feature gates decide whether bloom and color grading remain active. The history validation key and renderer frame receive only those resolved settings; raw `volume_stack` is cleared before renderer execution so concrete postprocess code cannot bypass the graph contract.

Graph attachment operations are now part of this submit boundary.
`RenderPipelineAsset` emits clear/load/store metadata by resource write order, and feature descriptors may explicitly override write ops for target initialization passes such as Deferred preview sky clearing imported `final-color`.
`RenderPassExecutionContext::attachment_ops_for_write(...)` exposes that decision to executors, and graph-owned executors translate the neutral contract through `scene_renderer::attachment_ops`.
`RenderPassExecutionContext` stays the graph metadata and resource-access owner, while the renderer-heavy GPU payload, mesh draw list view, post-process stack context, and concrete `record_*` bridge methods live in `render_pass_execution_context/gpu.rs`.
`RenderGraphExecutionResources::materialize_transient_resources(...)` backs live dense transient graph textures/buffers with concrete WGPU resources before pass execution, preserves imported frame targets, and deliberately skips sparse-reserved textures so virtual resources cannot be mistaken for dense allocations.
`sky.preview-scene-color` and `sky.preview-final-color` execute in the `DepthPrepass` stage before geometry, clear the background target plus `scene-depth`, and then let depth/opaque passes load the initialized attachments.
Depth/normal prepass consumes both the `scene-depth` and `gbuffer-normal` write ops before calling `NormalPrepassPipeline::record_with_attachment_ops(...)`.
`shadow.map` requires the graph-bound `shadow-map` depth texture and opens a depth-only render pass with the descriptor's clear/store ops, keeping the Shadow stage visible in captures and resource diagnostics.
`render_framework_stats_report_shadow_map_graph_execution` proves a submitted WGPU RenderFramework frame reports the executed `shadow-map` pass, `shadow.map` executor id, graph debug marker, and `last_shadow_graph_executed_pass_count` while later caster drawing and lighting sampling remain explicit follow-up work.
`ao.ssao-evaluate`, `lighting.clustered-cull`, and `post.bloom-extract` require post-process stack context and write graph-bound `ambient-occlusion`, `light-list`, and `bloom-texture` resources at their declared stages.
`post.stack` consumes those resources plus graph-bound `scene-depth`, scene/final/GI targets, and the clustered light-list buffer for final post-process recording only, replacing the former fixed `execute_post_process_stack(...)` wrapper without hiding preparation work inside one pass.
`ui.screen-space` passes the `viewport-output` write ops into `ScreenSpaceUiRenderer::record(...)` before it opens the WGPU render pass.
`overlay.gizmo` requires the graph-bound `viewport-output` and `scene-depth` resources plus prepared overlay buffers before calling `ViewportOverlayRenderer::record_overlays(...)`, so compiled-scene submission no longer draws overlays through a private post-graph call.
Sprite executors now pass separate `scene-color` and `scene-depth` ops into `SpriteRenderer::record(...)`, making the Core2d opaque sprite pass the depth producer and preserving the old Core2d placement by calling graph stages from the compiled-scene main pass order.
Forward mesh executors pass `scene-color` ops into `BaseScenePass`, require mesh draw/pipeline context plus `scene-depth`, and split prepared draws into opaque, alpha-mask, and transparent buckets.
Deferred executors pass `gbuffer-albedo`, `gbuffer-material`, and `scene-color` attachment ops into the concrete G-buffer and lighting passes, and `import_frame_targets(...)` binds `gbuffer-albedo`, `gbuffer-material`, and `gbuffer-normal` from the offscreen target set.
The fixed legacy `render_scene(...)` path remains explicit `Load + Store`; new graph-owned renderers should consume pass metadata rather than copy pass-name-specific load/store rules.

M8 compute/storage writes are now separated from attachment writes at the same SRP boundary. `RenderFeatureResourceWriteMode` lets descriptors declare storage outputs that compile to `write_storage_texture(...)` or `write_storage_external(...)`, keeping dependency, culling, queue-lane, and debug-marker evidence without assigning attachment load/store ops. `ao.ssao-evaluate` reads graph `scene-depth` and `gbuffer-normal`, writes `ambient-occlusion` as storage external output, and still declares `QueueLane::AsyncCompute`; when async compute is unavailable, pipeline compile falls it back to the graphics execution queue while preserving the declared async pass count and the no-attachment-ops resource metadata. Descriptor validation rejects storage write mode on read resources and rejects attachment ops on storage writes.

M8 compute workload planning now lives at the descriptor and RenderGraph compile boundary before any renderer-owned GPU object exists. `RenderGraphComputeWorkload` records only a pipeline label, workgroup size, and dispatch extent (`Viewport`, `ClusterGrid`, or fixed groups). `RenderFeaturePassDescriptor::with_compute_workload(...)` lets SRP features attach that plan to a pass; `RenderPipelineAsset::compile(...)` validates that it is declared only on `AsyncCompute` passes with a non-empty pipeline label and non-zero workgroup size, then preserves it on the compiled graph pass. The built-in SSAO and clustered-lighting descriptors declare viewport and cluster-grid workload plans through shared descriptor constants. This is planned metadata for pipeline activation, review, and future plan-vs-execution diagnostics; it is not a WGPU pipeline, bind group, texture, or proof that a dispatch already ran.

The same M8 path now records concrete compute dispatch evidence after the graph executor resolves GPU resources. `RenderGraphComputeDispatchRecord` stores the graph pass name, executor id, renderer-private pipeline label, workgroup size, `[x, y, z]` dispatch group count, and storage-write resource names; the record intentionally does not expose WGPU device, pipeline, bind group, or texture handles. `record_ssao_to_resources(...)` pushes a `zircon-ssao-pipeline` dispatch only when SSAO is runtime-enabled, and `record_clustered_lighting_to_resources(...)` pushes a `zircon-cluster-pipeline` dispatch only when clustered lighting is runtime-enabled. Disabled fallbacks may still clear/write their output buffers, but they do not count as compute dispatches. `execute_graph_stage(...)` drains those GPU-context records into `RenderGraphExecutionRecord`, builds a dispatch audit context from the viewport size and derived cluster grid, and audits them against the compiled pass `RenderGraphComputeWorkload`. The audit compares pipeline label, workgroup size, and planned-versus-actual dispatch groups for `Viewport`, `ClusterGrid`, and `Fixed` extents before publishing dispatch and audit counts through `RenderStats.last_graph_compute_*` plus the `render.graph.compute_dispatch_count`, `render.graph.compute_dispatch_group_count`, `render.graph.compute_storage_write_resource_count`, `render.graph.compute_planned_workload_count`, `render.graph.compute_matched_workload_count`, `render.graph.compute_missing_dispatch_count`, `render.graph.compute_workload_mismatch_count`, and `render.graph.compute_unexpected_dispatch_count` diagnostics rows. Async fallback tests can therefore prove a real compute pass body launched and matched the SRP plan without leaking backend objects across the framework boundary.

Renderer-side queue preparation is also surfaced at submit scope. `SceneRendererCompiledSceneOutputs` carries mesh queue stats from `prepare_mesh_queue(...)` and sprite queue stats from `prepare_sprite_queue_stats(...)` back to the render framework. `RenderStats.last_mesh_*` records draw/phase counts, early-z eligibility, prepared versus dynamic geometry, indirect draws, and static/dynamic/GPU-instancing candidate groups. `RenderStats.last_sprite_*` records draw batch count, batched sprite count, generated vertex count, and per-phase sprite batch counts. These counters prove what the graph-owned renderer prepared without exposing WGPU buffers or forcing the future mesh allocator and Bevy-like sprite binning work to land first.

Submit safety is guarded by viewport generations. Context building captures the viewport record generation while resolving size, effective pipeline, quality profile, and history state. Before runtime prepare mutates viewport runtime state, and again before recording the rendered frame back into the viewport, submit revalidates that the viewport still exists and that its generation matches. Missing viewports return `RenderFrameworkError::UnknownViewport`; changed viewports return `RenderFrameworkError::ViewportChanged` instead of relying on checked-then-`expect` panics.

Frame history reuse now includes an extract validation key and an explicit submit status. `build_frame_submission_context(...)` records world id, camera snapshot, mesh identity/transform/model/material/tint/mobility/layer mask, lighting extract, animation pose extract, post-process settings, particle extract, and the compiled effective feature names. It also carries both target size and render size so dynamic resolution cannot reuse temporal history just because the swapchain or imported viewport target stayed the same. `resolve_history_handle(...)` reuses the previous `FrameHistoryHandle` only when target size, render size, pipeline, history bindings, and this validation key all match. Camera motion, mesh motion, material/tint/layer changes, light changes, pose changes, bloom/color-grading/preview changes, particle changes, world changes, feature toggles, and dynamic render-scale changes therefore allocate a new history handle before renderer history textures are reused.

The compatibility result is carried as `FrameHistoryStatus` in `RenderStats.last_frame_history_status`. The status reports the current handle, previous handle, whether the previous frame was actually usable, target size, render size, and the invalidation reason (`no_previous_frame`, `viewport_resized`, `render_size_changed`, `pipeline_changed`, `history_binding_changed`, or `frame_inputs_changed`). The renderer submit path receives this result as `previous_history_available`, so `prepare_history_textures(...)` no longer treats an existing same-size texture as sufficient proof that temporal history is valid. Renderer history copy also preserves slot semantics: `FrameHistorySlot::SceneColor` is copied from `OffscreenTarget.scene_color`, not from post-processed `final_color`, so bloom/color grading and later overlay/UI composition do not feed back as scene-color history.

The graph-facing resource names are split by temporal role. `history.previous.scene-color` is the compatible previous texture imported for history resolve; `history.current.scene-color` labels the renderer-owned texture that `HistoryCopy` updates after the frame; `postprocess.history-resolved` is the output consumed by final composition. The built-in `HistoryResolve` descriptor reads `scene-color` and `history.previous.scene-color`, then writes `postprocess.history-resolved`; the old single `history-scene-color` name is intentionally not a compiled graph resource.

The effect-stack resource name follows the same explicit-output rule. `postprocess.effect-stacked` is produced only by the authored or volume-resolved `EffectStack` product node and becomes the sole final-composite input when any non-default effect-stack family is active. Default submissions keep the old single final-composite path and do not add a skipped placeholder node.

Graphics debugger capture is a submit-scoped request, not a persistent rendering mode. The only live triggers are `RenderFramework::request_graphics_debugger_capture(viewport)` and `ZR_RENDERDOC_CAPTURE_NEXT=1`; editor UI and dynamic API commands do not currently expose a separate RenderDoc button. The trait method stores a pending viewport in `WgpuRenderFramework`; non-matching viewport submits leave it pending. The environment variable arms the first viewport created by the framework so desktop debug launches can capture the first rendered frame without editor code calling the trait method. On the matching submit, capture begins before runtime prepare/render command recording and finishes after the frame is produced. The blocking wgpu stop/poll step runs after the framework state mutex is released, while an operation lock remains held so no second frame or viewport/pipeline mutation can enter the active capture window. Destroying a viewport with pending or queued debugger capture clears that debugger state and records a destroyed-viewport error instead of leaving `capture_pending` true forever. The status query reports wgpu capture-hook availability, the selected wgpu backend as `wgpu(dx12)` / `wgpu(vulkan)` / equivalent, pending/active flags, the last captured frame generation, and any submit or stop error. `available` means the backend exposes the wgpu debugger capture hook; it does not prove RenderDoc is attached. If the matching submit fails during preflight before capture starts, the pending request is consumed and `last_error` records the preflight error. If a submit fails while capture is active, cleanup still stops the capture and clears active/pending state before returning the original error.

RenderDoc-readable markers are centralized in `zircon_runtime/src/graphics/debug_markers.rs`. The compiled-scene command encoder emits markers for `FrameExtract`, `Clear`, `Prepass`, `MainScene`, `Lighting`, `DeferredLighting`, `PostProcess`, `HistoryCopy`, `Overlay`, and `UI`; the readback path emits `Readback` before the GPU-to-CPU copy. Graph-stage execution maps `RenderPassStage::Lighting` to the generic `zircon::Lighting` marker so Forward+ lighting stages are not mislabeled as deferred, while the fixed deferred lighting pass still uses `zircon::DeferredLighting`. Each executed graph pass now also emits and records `zircon::RenderGraphPass::<pass-name>`, so compute-declared passes that fall back to the graphics queue still leave pass-level capture evidence and `RenderStats.last_graph_executed_debug_markers` can prove marker coverage without exposing WGPU encoder internals.

For Windows RenderDoc capture, launch Zircon from RenderDoc with environment variables set before process start: use `WGPU_BACKEND=dx12` for Direct3D 12 or `WGPU_BACKEND=vulkan` for Vulkan, set `WGPU_DEBUG=1` and `WGPU_VALIDATION=1` when validation output is needed, and set `ZR_RENDERDOC_CAPTURE_NEXT=1` to capture the first created viewport's next submit. After capture, inspect the event browser for `zircon::FrameExtract`, `zircon::MainScene`, `zircon::Lighting`, `zircon::PostProcess`, `zircon::HistoryCopy`, and `zircon::UI`; history textures are labeled `history.current.scene-color`, `zircon-history-global-illumination`, and `zircon-history-ambient-occlusion`. The CPU fallback path still marks final readback as `zircon::Readback`, while the app-host window path now binds a native surface and finishes redraw through a wgpu swapchain `SurfaceTexture::present()` after a `zircon-present-blit-pass`. Keep `HistoryResolve` explicitly disabled unless the test scenario intentionally opts into temporal scene-color blending.

Validation coverage lives in `render_product_pipeline`, `render_product_submit`, `render_product_shadows`, `pipeline_compile`, `project_render`, `render_framework_bridge`, and `render_debugger_and_history` tests. The submit test intentionally verifies that direct extract frames can diverge from the legacy scene snapshot, proving product rendering must not use `to_scene_snapshot()` as the draw authority. The 2026-06-02 preview-sky/prepass/Deferred/post-process/UI-overlay graph cutover evidence used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib --locked pipeline_compile --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 43 SRP compile tests; `cargo test -p zircon_runtime --lib --locked graph_execution --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 34 graph-execution tests; `cargo test -p zircon_runtime --lib --locked render_product_submit --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 11 submit tests; `cargo test -p zircon_runtime --lib --locked render_framework_stats_report_executed_render_graph_passes --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the graph execution stats regression; and `cargo test -p zircon_runtime --lib --locked render_product_post_process --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 10 post-process tests. The 2026-06-03 M8 workload-audit slices passed `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`, `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never compute_workload`, `headless_wgpu_server_falls_back_async_compute_passes_to_graphics`, and `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins`; existing warnings only. The latest dispatch-extent follow-up extends the compute-workload tests to cover viewport, cluster-grid, fixed, and dispatch-group mismatch evidence. The neural compute slot follow-up reused the generated runtime test binary after Cargo wrapper timeouts during compile/link and passed `neural_compute` (3 filtered tests), `flagship_feature_descriptors_declare_backend_capability_requirements`, `default_pipeline_assets_do_not_embed_pluginized_advanced_builtin_features`, `compiled_pipeline_capability_validation_reports_neural_compute_requirement`, and `renderer_data_document_accepts_neural_compute_builtin_feature_source`; `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings. The neural compute activation follow-up added bridge coverage for missing `plugin.neural.inference` executor registration and missing backend `neural_compute` capability; direct execution of `zircon_runtime-b34ee8d8fc52f1fd.exe render_framework_rejects_neural_compute --nocapture` passed 2 bridge tests and direct execution of `zircon_runtime-b34ee8d8fc52f1fd.exe neural_compute --nocapture` passed 6 filtered tests after the Cargo wrapper timed out during compile/link, with the same scoped `cargo check` passing with existing warnings. The advanced follow-up slot slice passed `rustfmt --edition 2021 --check`, direct binary `advanced_followup --nocapture` with 3 focused tests, direct binary `pipeline_compile --nocapture` with 48 SRP compile tests, and the scoped `cargo check`; its first Cargo test wrapper timed out during compile/link after required target-dir cleanup, then the generated binary completed successfully. The executor registry module split was validated with `cargo test -p zircon_runtime --lib render_pass_executor_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`, which passed 27 registry and graph-executor tests after moving registry tests into the folder-backed test module. The 2026-06-04 shadow graph execution/materialization slice passed `cargo test -p zircon_runtime --lib shadow_map_pass_stays_live_as_depth_only_graph_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`, `cargo test -p zircon_runtime --lib shadow_map_executor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`, and a corrected `cargo test -p zircon_runtime --lib materialization_creates_dense_transients_and_skips_sparse_reservations --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`. The materialization regression first failed because the authored unit pass was culled; the test was fixed to mark the pass side-effectful, matching the production `shadow-map` descriptor and preserving the rule that only live compiled lifetimes receive WGPU backing. The render debugger/history tests cover idle debugger status, exact DX12/Vulkan backend status under `WGPU_BACKEND`, backend env parsing, first-created-viewport capture arming, marker registry coverage, matching-viewport request consumption, unknown viewport rejection, destroyed pending-capture cleanup, history validation-key invalidation, and explicit history-resolve opt-in. Manual `.rdc` acceptance remains a desktop RenderDoc step because this automated gate cannot launch the GUI capture workflow.

The 2026-06-04 shadow submit telemetry and diagnostics follow-up first timed out while compiling the Windows runtime lib-test binary, then the warmed rerun `cargo test -p zircon_runtime --lib render_framework_stats_report_shadow_map_graph_execution --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test / 0 failed / 2683 filtered after adding the `last_shadow_graph_executed_pass_count` assertion. The diagnostics bridge rerun `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test / 0 failed / 2683 filtered, proving `render.graph.executed_shadow_pass_count`; existing warnings only.

The 2026-06-04 deferred material G-buffer slice passed scoped `rustfmt --edition 2021 --check` over the touched render graph/deferred/backend/test Rust files. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only, and `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only, proving the updated unit-test source compiles. A standalone Naga WGSL parse/validation check against `deferred_geometry.wgsl` and `deferred_lighting.wgsl` also passed. Two Cargo lib-test execution attempts for the fresh runtime test binary timed out or stalled during the Windows test-binary link step, so this slice does not claim a fresh `cargo test` execution result.

The 2026-06-05 SSR material roughness slice passed scoped `rustfmt --edition 2021 --check` over the touched postprocess/render Rust files, standalone Naga WGSL parse/validation for `post_process.wgsl`, and `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` with existing warnings only. This proves the binding-16 shader/layout/source-contract changes compile and validate; no fresh `cargo test` execution is claimed for this slice.

The 2026-06-05 SSR ray-march slice replaced the old one-sample reflected-coordinate seed with a bounded final-pass screen-space march over the existing depth/normal/material resources. The shader now caps SSR marching to 128 in-shader steps, uses authored max-step/max-distance/thickness settings, depth-tests candidate samples, fades by ray distance and screen edge, and keeps roughness attenuation. Scoped `rustfmt --edition 2021 --check`, standalone raw/fallback Naga WGSL parse/validation for `post_process.wgsl`, `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`, and focused `cargo test -p zircon_runtime --lib post_process_shader_ray_marches_ssr_with_bounds_and_edge_fade --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed with existing warnings only; no fresh broad `cargo test` execution is claimed for this slice.

The 2026-06-05 SSR projection follow-up added `PostProcessParams.effect_projection` as an internal WGPU uniform vector derived from the current camera and viewport. The shader now reconstructs the current pixel's view-space origin from camera-linearized depth, uses the view vector for reflection, marches in view-space distance, projects each candidate back to screen pixels, and keeps the same bounded depth/thickness/roughness/edge gates. Scoped `rustfmt --edition 2021 --check`, standalone raw/fallback Naga WGSL validation for `post_process.wgsl`, `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`, focused `cargo test -p zircon_runtime --lib effect_stack_settings_are_encoded_into_post_process_params --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`, and focused `cargo test -p zircon_runtime --lib post_process_shader_ray_marches_ssr_with_bounds_and_edge_fade --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed with existing warnings only.

The 2026-06-05 SSR hit-refinement follow-up keeps the final-pass SSR path bounded but improves the accepted hit. `post_process.wgsl` now shares the depth/thickness/distance/edge visibility calculation between the coarse march and a four-step `refine_screen_space_reflection_hit(...)` interval search, then samples the refined coordinate before blending. The raw and viewport-depth-fallback post-process shader tests now run Naga validation. `rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs` passed. `cargo test -p zircon_runtime --lib post_process_shader --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 8 shader tests, and `cargo test -p zircon_runtime --lib post_process_viewport_depth_fallback_shader_parses_for_gl_backends --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed the fallback shader regression. `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only.

The 2026-06-05 SSR normal-space follow-up keeps `gbuffer-normal` as a world-space graph resource for the existing normal prepass/deferred contract, but adds postprocess-only camera view-basis rows to `PostProcessParams`. The final-pass SSR shader now converts the sampled world normal through `effect_view_x/y/z` before reflecting the reconstructed view-space ray. Scoped `rustfmt --edition 2021 --check` passed over the touched postprocess Rust files. `cargo test -p zircon_runtime --lib camera_view_basis_is_encoded_for_post_process_normals --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed the rotated-camera uniform regression with existing warnings only. Direct execution of `E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe post_process_shader --test-threads=1 --nocapture` passed 8 shader source-contract tests including raw Naga validation and the updated SSR normal contract, and the same binary passed `post_process_viewport_depth_fallback_shader_parses_for_gl_backends --test-threads=1 --nocapture`.

The 2026-06-05 DoF lens/bokeh follow-up extends the neutral DoF settings and final-pass postprocess shader without claiming the full Unity-style CoC pipeline. `RenderDepthOfFieldSettings` now carries focus range, focal length, bokeh blade count, and bokeh rotation; volume resolution blends continuous lens values and treats blade count as discrete; `PostProcessParams.effect_dof_lens` uploads sanitized values; and `post_process.wgsl` uses a bounded rotated disk/bokeh kernel instead of the old four-direction cross blur. Validation used the isolated `E:\cargo-targets\zircon-dof-lens-0605` target dir after the shared render target hit a stale fingerprint path error. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-dof-lens-0605 --message-format short --color never` passed with existing warnings only. The first Cargo test wrapper timed out during Windows test-harness build/link, then the generated binary completed the focused direct runs: `effect_stack` passed 23 tests, `depth_of_field` passed 2 tests, `post_process_shader` passed 9 shader/source-contract tests with raw Naga validation, and `post_process_viewport_depth_fallback_shader_parses_for_gl_backends` passed 1 fallback shader regression. Scoped rustfmt, `git diff --check`, and conflict-marker scans passed with only expected line-ending warnings.

The 2026-06-05 DoF resource-contract follow-up reserves the split-pass resource vocabulary without claiming physical split passes. `PostProcessGraphResourceNames` now includes `postprocess.depth-of-field.coc` and `postprocess.depth-of-field.bokeh`; active DoF effect-stack nodes produce those outputs; and `import_frame_targets(...)` first imported them as renderer aliases so product-node resource validation could see the intended contract. The target-allocation follow-up replaces those aliases with concrete `OffscreenTarget` scratch textures and imports those views by the same names. Scoped `rustfmt --edition 2021 --check`, path-scoped `git diff --check`, and conflict-marker scans passed for the resource-contract slice. Clean Cargo validation is blocked in this checkout: focused test wrappers timed out in the Windows runtime lib-test compile/link path, the warmed `cargo check -p zircon_runtime --lib --tests --locked --target-dir E:\cargo-targets\zircon-dof-resource-check-0605` emitted fresh runtime metadata before the wrapper timeout, and a final rerun now fails immediately under `--locked` because unrelated active `Cargo.lock` changes for the first-party runtime catalog/plugin packages require lockfile regeneration.

The 2026-06-03 dynamic-resolution history-size follow-up passed scoped formatting, conflict-marker, and `git diff --check` scans over touched render/docs files, then passed `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir F:\cargo-targets\zircon-render-main-chain-history --message-format short --color never` with existing warnings. The focused `cargo test -p zircon_runtime --lib render_framework_invalidates_history_when_dynamic_render_size_changes --locked --jobs 1 --target-dir F:\cargo-targets\zircon-render-main-chain-history --message-format short --color never` attempt was blocked before test execution by unrelated active `static_manifest_contracts` test harness compile errors (`E0364` private re-exports and `E0282` inference errors). During validation, two unrelated UI test-harness compile blockers were mechanically unblocked: `dispatch_navigation_input(...)` route-policy annotation now ends the mutable borrow before reading route policy, and the untracked slider renderer test helper calls `state.disabled()` consistently.

The 2026-06-02 post-process graph record-owner slice used the same target dir: `stage_execution_records_post_process_graph_through_record_owner` passed 1 test, `render_product_post_process` passed 10 tests, `render_framework_stats_report_executed_product_postprocess_nodes` passed 1 test, and `graph_execution` passed 35 tests.

The 2026-06-02 M7 effect-stack parameter slice added focused regressions for `render_product_post_process_effect_stack_runs_before_final_composite_when_authored`, `render_product_post_process_extended_effect_stack_settings_enable_product_node`, `render_framework_stats_report_effect_stack_product_node_when_authored`, and the product-postprocess executor registry coverage for `post.effect-stack`. The follow-up M7 volume-stack slice added `render_framework_stats_report_volume_effect_stack_product_node_when_authored` and `post_process/volume.rs` unit tests for priority/layer/local-blend and extended-effect resolution. The renderer consumption slice added `effect_stack_settings_are_encoded_into_post_process_params` to cover the uniform encoding consumed by `post_process.wgsl`; the camera-depth slice extended that bridge with near/far/projection params and added `orthographic_camera_depth_params_disable_perspective_linearization`. The report slice added `effect_stack_report_records_active_approximated_and_missing_resources` and extended the direct/volume bridge tests to assert `last_post_process_effect_stack_report` for active families, approximation labels, and missing LUT/SSR normal resources. The authored-LUT binding slice added `effect_stack_report_treats_authored_lut_as_renderer_bound_resource` plus enabled/disabled LUT resource-id filters in `resource_streamer_ensure_scene_resources.rs`, proving submit-resolved LUT handles drive renderer resource preparation and no longer count as approximation-backed. The LUT readiness/request slice added `color_lookup_intensity_requests_lut_even_without_texture_handle`, `effect_stack_lut_texture_request_tracks_enabled_lut_without_handle`, and `render.post_process.lut.*` diagnostics. The LUT shape-contract follow-up added `RenderColorLookupTextureLayout`, 2D-strip validation, explicit 3D shape classification, prepared texture descriptor retention, and shape diagnostic rows for `texture_2d_strip_ready_count`, `texture_3d_request_count`, and `unsupported_shape_count`. The 3D binding follow-up added dedicated post-process LUT texture resources, binding 12, a 2x2x2 identity cube fallback, shader parsing/sampling regressions for the 3D branch, and renamed the resource-streamer status test to `effect_stack_lut_texture_status_accepts_3d_lut_for_texture_3d_binding`. The 3D sampling follow-up added binding 13, a clamp/linear sampler, and a shader contract requiring `textureSampleLevel(...)` for the 3D branch while leaving 2D strip on `textureLoad(...)`. The `.cube` asset-ingress follow-up added `zircon.builtin.texture.cube_lut`, linear D3 RGBA8 `TextureAsset` output, parser metadata compatibility, explicit 1D shaper rejection, and importer capability coverage. Focused validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib post_process_shader_samples_bound_effect_lut_texture_3d --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 1 shader contract regression after the sampler-backed 3D branch change, `cargo test -p zircon_runtime --lib lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 19 LUT-filtered tests, the pre-shaper `cube_lut` run passed 5 cube-LUT tests, `cargo test -p zircon_runtime --lib texture_importer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 14 texture importer tests, `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 1 diagnostics regression, `cargo test -p zircon_runtime --lib render_framework_stats_report_effect_stack_product_node_when_authored --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 1 RenderFramework effect-stack stats regression, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings. A Windows shaper-guard rerun was blocked before test execution by a target-dir dep-info fingerprint path issue; the isolated WSL rerun `wsl -e sh -lc 'cd /mnt/e/Git/ZirconEngine && CARGO_TARGET_DIR=/tmp/zircon-render-main-chain-cube-lut-0603 cargo test -p zircon_runtime --lib cube_lut --locked --jobs 1 --message-format short --color never'` passed 6 cube-LUT tests, 2500 filtered out, with existing warnings only.
