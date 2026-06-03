---
related_code:
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/Volume.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
  - dev/Graphics/com.unity.postprocessing/PostProcessing/Editor/Tools/CubeLutAssetImporter.cs
  - dev/Graphics/com.unity.postprocessing/PostProcessing/Editor/Tools/CubeLutAssetFactory.cs
  - dev/Graphics/com.unity.postprocessing/PostProcessing/Editor/Resources/ComputeShaders/Lut3DBaker.compute
  - dev/bevy/crates/bevy_post_process/src/lib.rs
  - dev/bevy/crates/bevy_post_process/src/bloom/mod.rs
  - dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs
  - dev/bevy/crates/bevy_post_process/src/motion_blur/mod.rs
  - dev/bevy/crates/bevy_post_process/src/dof/mod.rs
  - dev/bevy/crates/bevy_post_process/src/msaa_writeback.rs
  - dev/bevy/crates/bevy_core_pipeline/src/tonemapping/lut_bindings.wgsl
  - dev/bevy/examples/3d/post_processing.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/volume.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/fallback/create_fallback_texture.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/importer/ingest/import_cube_lut.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/mod.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_post_process_lut_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_post_process_lut_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/bloom.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/history_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs
  - zircon_runtime/src/core/framework/render/post_process/volume.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/fallback/create_fallback_texture.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/importer/ingest/import_cube_lut.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/mod.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_post_process_lut_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_post_process_lut_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/bloom.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/history_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
plan_sources:
  - user: 2026-05-16 continue Render M4B postprocess pass graph productization
  - user: 2026-05-18 continue Render M8A anti-alias product surface
  - user: 2026-05-20 continue Bevy-level render postprocess evidence mapping
  - user: 2026-05-22 continue M10 post-process and anti-alias breadth checklist
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - user: 2026-06-03 implement ZirconEngine WGPU render main-chain closure plan, M7 LUT asset ingress slice
  - user: 2026-06-03 implement ZirconEngine WGPU render main-chain closure plan, M7 SSR normal and depth-backend fallback slice
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/framework/tests.rs::render_product_post_process_effect_stack_runs_before_final_composite_when_authored
  - zircon_runtime/src/core/framework/tests.rs::render_product_post_process_extended_effect_stack_settings_enable_product_node
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests
  - zircon_runtime/src/core/framework/render/post_process/volume.rs::tests
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - tests/acceptance/render-product-m4b-post-process.md
  - cargo test -p zircon_runtime --locked render_product_post_process
  - cargo test -p zircon_runtime --locked render_product_anti_alias
  - cargo test -p zircon_runtime --locked render_graph
  - zircon_runtime/src/core/framework/tests.rs::render_product_post_process_stack_splits_history_previous_and_output_slots
  - zircon_runtime/src/core/framework/tests.rs::render_product_post_process_effect_stack_runs_before_final_composite_when_authored
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_records_active_approximated_and_missing_resources
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::color_lookup_intensity_requests_lut_even_without_texture_handle
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_treats_authored_lut_as_renderer_bound_resource
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::color_lookup_texture_layout_accepts_current_2d_strip_contract
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::color_lookup_texture_3d_layout_is_recognized_but_not_2d_bindable
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_records_invalid_lut_layout_size
  - zircon_runtime/src/core/framework/render/post_process/effect_stack_settings.rs::tests::effect_stack_report_treats_bound_ssr_normal_as_available
  - zircon_runtime/src/core/framework/render/post_process/stack.rs::tests::effect_stack_ssr_declares_depth_and_normal_inputs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_id_uses_enabled_lookup_handle
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_id_ignores_disabled_lookup_handle
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::effect_stack_lut_texture_request_tracks_enabled_lut_without_handle
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
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/effect_lut_texture_view.rs::tests::generated_effect_lut_is_s_curve_with_stable_texture_stride
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_fallback_texture_views/effect_lut_texture_view.rs::tests::generated_effect_lut_3d_is_identity_cube
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_parses_after_lut_binding_expansion
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_effect_lut_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_effect_lut_texture_3d
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_scene_depth_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_scene_normal_texture_for_ssr
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_viewport_depth_fallback_shader_parses_for_gl_backends
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/depth_sampling_mode.rs::tests::viewport_depth_fallback_shader_removes_raw_depth_texture_sampling
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::effect_stack_resource_status_detects_graph_bound_ssr_normal
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::builtin_registry_covers_product_postprocess_executor_ids
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::history_resolve_compiles_only_with_explicit_feature_opt_in
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - cargo test -p zircon_runtime --lib cube_lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib texture_importer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Postprocess Pass Graph Contracts

## Purpose

`zircon_runtime::core::framework::render::post_process` owns the neutral M4B postprocess product contract. It describes the per-camera stack as data on `RenderFrameExtract` and keeps effect ordering, resource names, and validation independent from WGPU pipelines.

Concrete rendering stays in `zircon_runtime::graphics`. The renderer consumes the frame's validated graph for execution evidence and treats actual renderer history availability as a final resource gate: when history is not available, it derives a frame-local graph by removing only the history resource/node from the submitted stack. It still uses the existing shader-backed postprocess stack for pixels until later milestones replace individual passes with graph-native implementations.

## Bevy Evidence

The Bevy reference surface is `bevy_post_process`, not a single monolithic effect pass. `dev/bevy/crates/bevy_post_process/src/lib.rs:9-36` splits the crate into `auto_exposure`, `bloom`, `dof`, `effect_stack`, `motion_blur`, and `msaa_writeback`, then wires `MsaaWritebackPlugin`, `BloomPlugin`, `MotionBlurPlugin`, `DepthOfFieldPlugin`, and `EffectStackPlugin` into `PostProcessPlugin`.

`dev/bevy/crates/bevy_post_process/src/bloom/mod.rs:44-83` shows bloom as a real Core2d/Core3d post-process system with extracted component data, prepared textures/bind groups, and scheduling before tonemapping. `dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs:3-6` names the built-in effect-stack features as chromatic aberration and vignette, while `effect_stack/mod.rs:141-165` extracts those camera components and schedules the combined pass before tonemapping.

The heavier Bevy effects have additional prerequisites. `dev/bevy/crates/bevy_post_process/src/motion_blur/mod.rs:75-173` requires depth and motion-vector prepasses and runs in the Core3d post-process set before bloom. `dev/bevy/crates/bevy_post_process/src/dof/mod.rs:69-241` defines depth-of-field camera state, prepares depth/focus resources, and schedules after bloom and before tonemapping. `dev/bevy/crates/bevy_post_process/src/msaa_writeback.rs:21-33` registers MSAA writeback before the Core2d/Core3d main pass, and `msaa_writeback.rs:110-126` only inserts the blit pipeline when camera writeback policy and MSAA sample count require it.

The user-facing Bevy example `dev/bevy/examples/3d/post_processing.rs` demonstrates the effect-stack authoring model by attaching chromatic aberration and vignette components to a 3D camera. Zircon does not yet expose equivalent camera components for those effects.

## Unity Volume Evidence

The volume-stack side of M7 follows Unity SRP more closely than Bevy. `dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/Volume.cs`, `VolumeProfile.cs`, `VolumeManager.cs`, and `VolumeStack.cs` establish the same product concepts Zircon now mirrors in neutral form: profiles contain optional effect overrides, volumes can be global or local, priority and blend weight decide ordering and influence, and camera layer masks decide which authored volumes are relevant to the current view.

Zircon intentionally stops below Unity's full spatial collider evaluation in this slice. Scene extract or a future scene-volume system computes local influence and writes it as `RenderPostProcessVolume.local_blend`; the render submit path only performs deterministic filtering, ordering, and parameter blending before the graph is built.

Unity also sets the practical LUT import boundary for this M7 slice. `CubeLutAssetImporter.cs` parses `.cube` headers into a square 3D LUT, rejects sizes outside 2-256, and creates a clamped bilinear `Texture3D`; `CubeLutAssetFactory.cs` uses 33 as the default authoring size. Zircon mirrors that as neutral metadata in `RenderColorLookupTextureLayout` instead of binding a WGPU type at the framework boundary. `Texture2dStrip { size }` remains the 2D-strip layout for authored LUTs that arrive as atlas-style texture assets, while `Texture3d { size }` now routes through renderer-private WGPU `texture_3d` upload and final post-process shader sampling with a clamp/linear sampler. Bevy's `tonemapping/lut_bindings.wgsl` is the reference for the renderer-private 3D texture plus sampler binding shape, not something `app`, `editor`, or `framework` may import directly. The basic `.cube` asset ingress now lives in `zircon_runtime::asset`: `.cube` text imports become linear RGBA8 `TextureAsset` payloads with a neutral D3 descriptor, then renderer preparation consumes them through the same authored 3D LUT path as any decoded texture asset.

## Data Model

`PostProcessEffectKind` currently names the first product nodes: bloom, color grading, history resolve, effect stack, final composite, and FXAA. These names are intentionally product-level, not tied to a specific shader or render-pass asset.

`PostProcessEffectSettings` is the authored node descriptor. It carries the effect kind, enabled flag, required input resource names, produced output resource names, and `after` ordering dependencies.

`PostProcessStackDescriptor` stores the initial resource set and ordered authored effects. `from_extract_settings(...)` derives the default stack from `RenderBloomSettings`, `RenderColorGradingSettings`, history-resolve enablement, and history availability. History resolve requires both an enabled profile feature and a compatible previous history texture; the first compatible frame keeps the history node skipped until renderer history is actually available. Disabled effects remain visible in the descriptor but are elided from executable graph nodes.

`RenderPostProcessEffectStackSettings` is the M7 typed parameter bundle for effect-stack style postprocess. It currently carries neutral settings for tonemap, color lookup table, blur, depth of field, screen-space reflection, vignette, film grain, dither, chromatic aberration, and fog. `RenderColorLookupSettings` includes `RenderColorLookupTextureLayout`, giving authored LUTs an API-level shape contract before WGPU resources are selected. Positive LUT intensity is the authored request signal even when no texture handle is present; missing handles and invalid explicit LUT sizes are reported by the effect-stack report. A default bundle is treated as absent so existing default graphs do not grow a disabled placeholder node. When any setting is active, `PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(...)` inserts an `EffectStack` product node after bloom/color-grading/history inputs and before final composite, producing `postprocess.effect-stacked`.

This parameter bundle is still a render-chain contract, not full shader parity. The renderer now encodes effect-stack fields plus camera depth parameters into `PostProcessParams`, and `post_process.wgsl` consumes those uniforms for baseline tonemap/exposure, authored-or-fallback LUT texture sampling, blur, camera-linearized scene-depth DoF-radius sampling, normal-seeded and depth-gated SSR seed blending, vignette, grain, dither, chromatic aberration, and view-depth-weighted fog. `RenderColorLookupSettings::is_enabled()` treats positive intensity as the LUT request signal; the optional texture handle decides whether the WGPU postprocess bind group selects a prepared 2D/2D-strip LUT at binding 10, a prepared 3D LUT at binding 12, or renderer-owned fallbacks. Binding 10 keeps the 64x1 S-curve fallback for 1D/2D-style LUT use, binding 12 keeps a 2x2x2 identity cube fallback for `Texture3d`, binding 13 carries the clamp/linear sampler used by the 3D path, and `PostProcessParams.effect_flags.y` records the actual shader binding mode selected for the frame: disabled, 2D, 2D strip, or 3D. The 3D shader path samples normalized texel-center coordinates with `textureSampleLevel(...)`, preserving endpoint colors while allowing interpolation between LUT cells. The renderer inspects the neutral `RenderImageDescriptor` before upload, so unsupported array/non-LUT shapes are diagnosed and kept on fallback instead of being forced through the wrong WGPU slot. Binding 11 carries graph-owned `scene-depth` on backends that can compile the raw depth shader path, while `PostProcessDepthSamplingMode` switches GL/WebGL/ANGLE backends to a viewport-depth fallback shader and a regular float fallback texture because Naga's GLSL path translates raw depth sampling into a shadow-sampler shape. Binding 14 carries graph-owned `gbuffer-normal`, and the SSR seed reflects the view vector by the decoded normal before applying depth matching. `PostProcessParams.effect_depth` carries clamped near/far planes, inverse range, and projection-mode flag so perspective and orthographic cameras interpret sampled or fallback depth consistently. Basic text `.cube` import is now asset-pipeline work that feeds this 3D path; half-float LUTs, 1D shaper LUTs, editor baking, a real focus/lens model, roughness-backed ray-marched SSR, a linear-depth color resource for GL fallback quality, and split per-effect passes remain future renderer or authoring work.

`RenderPostProcessEffectStackReport` is the public diagnostic companion for that intentionally partial state. It is derived from the effective settings after direct extract and volume resolution, records active effect families, records which active families are still approximation-backed (`depth-of-field`, `screen-space-reflection`), and exposes missing authored-resource labels such as `effect-stack.lut.texture`, `effect-stack.lut.texture-layout`, and `effect-stack.ssr.normal`. The LUT label now means intensity was authored without a LUT handle; once a handle is present, LUT is considered renderer-bound and no longer counted as approximation-backed. Renderer-private upload failure still falls back to the S-curve or identity-cube texture at bind time rather than changing the neutral report, while renderer-side `RenderStats.last_post_process_lut_*` counters expose whether the enabled LUT family requested a texture, prepared it, used fallback, matched the current 2D-strip contract, requested and prepared a 3D layout, or had an unsupported shape. DoF/SSR depth is no longer reported as missing because `post.stack` now binds graph `scene-depth`. SSR normal readiness is now graph-resource-driven: when the effective effect-stack node declares `gbuffer-normal`, submit stats pass `RenderPostProcessEffectStackResourceStatus { ssr_normal_available: true }` and the old `effect-stack.ssr.normal` missing label is cleared. The report uses only count and string fields so `RenderStats` remains cloneable and equality-friendly for focused bridge tests.

`RenderPostProcessVolumeProfile`, `RenderPostProcessVolume`, and `RenderPostProcessVolumeStack` are the M7 volume authoring DTOs carried by `PostProcessExtract`. A profile can override bloom, color grading, and the effect-stack settings independently. A volume records active/global state, priority, weight, render-layer mask, local blend influence, and the profile payload. `RenderPostProcessVolumeStack::resolve(...)` filters volumes by active state, camera layer intersection, and positive influence, then blends them in ascending priority order with insertion-order tie breaking. Global volumes use only `weight`; local volumes use `weight * local_blend`, where `local_blend` is intentionally precomputed by scene extract rather than calculated in the renderer.

`RenderResolvedPostProcessSettings` is the resolved per-camera output of that stack. It is not stored as authored scene state; it exists to feed the submit-time graph compiler and history-validation key with the final bloom, color-grading, and effect-stack settings for the current camera.

History scene-color is deliberately split into previous, current, and output names. The previous slot is the only graph input imported from the last accepted frame. The current slot names the renderer-owned texture that `HistoryCopy` updates at the end of a valid frame. The output slot is the postprocess result produced by the history-resolve pass for later composition. A pass must not use one shared resource name for both previous input and current output, because that hides temporal overwrite bugs from graph validation.

`PostProcessPassGraph` is the validated graph summary carried by `PostProcessExtract`. It contains executable nodes, skipped nodes, and the final composite node name for stats and tests.

## Resource Names

`PostProcessGraphResourceNames` defines the stable resource vocabulary used by neutral contracts and concrete renderer resource import:

- `scene-color`
- `scene-depth`
- `gbuffer-albedo`
- `gbuffer-normal`
- `ambient-occlusion`
- `global-illumination`
- `light-list`
- `history.previous.scene-color`
- `history.current.scene-color`
- `postprocess.history-resolved`
- `bloom-texture`
- `postprocess.color-graded`
- `postprocess.effect-stacked`
- `postprocess.final-composited`
- `final-color`

The compiled-scene renderer imports the physical offscreen target textures and buffers under these declared names. `scene-color`, `scene-depth`, `gbuffer-*`, `ambient-occlusion`, `global-illumination`, `final-color`, and `bloom-texture` map to concrete frame target resources; `light-list` maps to the clustered lighting buffer. `postprocess.color-graded`, `postprocess.effect-stacked`, `postprocess.history-resolved`, and `postprocess.final-composited` are imported aliases for graph-resource consistency while concrete shader execution still writes through the existing postprocess target set. `history.previous.scene-color` is imported only when `prepare_history_textures(...)` reports a compatible previous history texture, so execution evidence cannot claim history resolve on the first frame after allocation or rotation. `history.current.scene-color` is the label of the renderer history texture written by `HistoryCopy`; it is not the graph input consumed by history resolve.

## Validation

`PostProcessPassGraph::validate_stack(...)` enforces the graph invariants before execution evidence is recorded:

- enabled nodes must have all required inputs available before they run,
- produced outputs must not duplicate initial resources or another enabled node output,
- `after` dependencies must target an enabled node in the graph,
- dependency cycles reject the stack before renderer execution.

Disabled effects are not errors. They are converted into `skipped_nodes` so stats and diagnostics can distinguish an authored but disabled effect from a node that never existed.

## Runtime Submit Integration

`build_frame_submission_context(...)` derives the effective stack from the compiled feature set, extract settings, compatible frame history, resolved volume stack, and resolved anti-alias settings. It first calls `PostProcessExtract::resolved_settings_for_layers(...)` with the current camera render layers, then applies feature gates. Profile-disabled bloom or color grading is converted back to default settings before graph validation, history resolve only enters the graph when the compiled pipeline enables the history feature and the viewport already has compatible frame history, and FXAA enters the graph only when `AntiAliasSettings` resolves to `Fxaa`.

`FrameSubmissionContext` carries the effective bloom settings, color-grading settings, effect-stack settings, `PostProcessStackDescriptor`, and `PostProcessPassGraph`. Extract-submit, present-submit, and direct runtime-frame submit replace the frame's stack and graph with those effective values and clear the raw `volume_stack` before calling the renderer, so renderer execution starts from the active pipeline rather than raw authored settings.

The compiled-scene renderer is the final authority for execution evidence. The submitted effective graph remains the source graph for the frame, but after `prepare_history_textures(...)` reports actual renderer history availability, the renderer records a frame-local `PostProcessExtract` graph through `RenderGraphStageExecution::record_post_process_graph(...)`. If history is unavailable, that frame-local graph is derived from the submitted validated stack by dropping the history resource and disabling only the history-resolve node. `RenderStats` reads node counts/final-composite metadata from the renderer graph when available. This keeps stats and executed-node evidence aligned when viewport metadata says history is compatible but the concrete history texture was just allocated, released, or resized. The compiled-scene root does not directly pair post-process graph resources with the execution record; that ownership stays inside the graph-stage execution context.

The concrete built-in graph executors now split preparation and final composition by declared pass. `ao.ssao-evaluate` records SSAO into `ambient-occlusion`, `lighting.clustered-cull` fills `light-list`, and `post.bloom-extract` records bloom into `bloom-texture`. `post.stack` no longer dispatches those three preparation passes; it consumes graph-bound `scene-depth`, `gbuffer-normal`, scene color, AO, bloom, final color, GI target, and light-list resources for the final post-process composite. Missing post-process stack context is a hard executor error rather than a silent no-op.

The product-node executors for `post.bloom`, `post.color-grading`, `post.history-resolve`, `post.effect-stack`, `post.final-composite`, and `post.fxaa` still validate the submitted product post-process node resources through `RenderGraphExecutionResources`. They remain evidence/metadata executors for the neutral product graph, while the WGPU pixel work is owned by the pass executors above. Effect-stack pixels currently run through the final post-process shader's uniform block rather than separate graph-native passes per effect family.

`RenderStats` reports `last_post_process_graph_node_count`, `last_post_process_graph_skipped_node_count`, `last_post_process_final_composite_node`, `last_post_process_graph_executed_nodes`, `last_post_process_effect_stack_report`, `last_post_process_lut_{request,ready,fallback}_count`, `last_post_process_lut_2d_strip_ready_count`, `last_post_process_lut_3d_request_count`, and `last_post_process_lut_unsupported_shape_count`. The executed-node list is separate from normal render graph passes, so product postprocess evidence does not change existing pass-order expectations such as overlay staying the last compiled graph pass. The effect-stack report is derived from the same effective settings as the graph, so direct extract and volume-authored stacks produce comparable active-family and missing-resource diagnostics. `collect_runtime_diagnostics(...)` mirrors the report's enabled state, active-family count, approximation-backed count, and missing-resource count into `DiagnosticStore` under `render.post_process.effect_stack.*` paths, and mirrors LUT resource readiness and shape status under `render.post_process.lut.request_count`, `ready_count`, `fallback_count`, `texture_2d_strip_ready_count`, `texture_3d_request_count`, and `unsupported_shape_count` for runtime/editor tooling.

## Bevy Gap Classification

Zircon currently covers the neutral product graph, bloom/color-grading/history/effect-stack/final-composite node vocabulary, graph validation, renderer execution evidence, and the FXAA node that is shared with the anti-alias surface. That is enough for DefaultRender diagnostics and pass-order accountability.

Zircon is not yet Bevy-complete for post-processing. Motion blur is not implemented because the required motion-vector prepass contract has not been productized. Depth of field currently has a camera-near/far-linearized blur-radius approximation, but it still lacks a real focus/lens model plus auxiliary depth/blur resources exposed through `render::camera` or `render::post_process`. Chromatic aberration and vignette now have a neutral effect-stack DTO, graph/stat node, and baseline shader consumption, but they are not yet camera components or dedicated WGPU passes. MSAA writeback is represented only indirectly through camera MSAA settings and anti-alias fallback reporting; there is no Bevy-style sorted-camera MSAA writeback blit path yet.

The next Bevy-parity implementation milestone should add typed post-process authoring descriptors before adding more shader passes. The safe order is: camera-facing post-process settings, neutral graph nodes/resources, validation and stats, then concrete WGPU execution. This keeps advanced effects from bypassing the basic DefaultRender product contract.

## M10.6 Promotion Gate

M10.6 is the post-process side of the post-process/AA breadth gate. It does not treat the existing bloom, color grading, history resolve, final composite, or FXAA nodes as full Bevy post-process parity. Bevy's `PostProcessPlugin` installs MSAA writeback, bloom, motion blur, depth of field, and the chromatic-aberration/vignette effect stack as separate products, so Zircon promotion has to prove each family independently.

| Check | Current evidence | Promotion requirement |
| --- | --- | --- |
| Product graph stays family-aware. | `PostProcessStackDescriptor` and `PostProcessPassGraph` expose bloom, color grading, history resolve, effect stack, final composite, and FXAA nodes with graph validation and stats. | New effects must first add typed authoring descriptors, stable resource names, validation rules, skipped/executed stats, and pass-order diagnostics before renderer pixels are accepted. |
| Motion blur is not hidden behind post-process success. | Zircon has no productized motion-vector prepass contract for motion blur. | Add camera-facing motion-blur settings, depth/motion-vector prepass ownership, Core3d ordering before bloom, missing-prepass diagnostics, and focused tests. |
| Depth of field is not hidden behind bloom. | Zircon has no camera focus/lens model or auxiliary DoF texture resources in this contract. | Add focal/lens settings, Gaussian/bokeh mode vocabulary or an intentional narrower subset, auxiliary resource validation, and pass ordering after bloom before tonemapping-equivalent output. |
| Effect stack remains explicit. | Tonemap, LUT, blur, DoF, SSR, vignette, grain, dither, chromatic aberration, and fog now have neutral `RenderPostProcessEffectStackSettings`, an enabled-only graph node, `postprocess.effect-stacked`, executor registration, stats evidence, a Unity-style neutral volume stack that resolves into effective per-camera settings before graph validation, baseline WGPU uniform/shader consumption in the final post-process pass, authored-or-fallback LUT texture binding with request/ready/fallback diagnostics, 2D-strip LUT shape acceptance, renderer-private `texture_3d` binding plus clamp/linear sampling for decoded 3D LUT texture assets, basic `.cube` text import into linear 3D RGBA8 `TextureAsset`, graph `scene-depth` binding, backend-gated GL depth fallback, graph `gbuffer-normal` binding for SSR seed direction, and camera near/far linearization for DoF/SSR/fog baselines. | Add scene-authored camera/volume components, advanced LUT authoring policy such as shaper/half-float/baked LUTs, focus/lens/bokeh DoF resources, roughness-backed ray-marched SSR, a linear-depth color resource to replace the GL viewport-depth fallback, per-effect prepare diagnostics, and graph-native split passes where needed before claiming the effect-stack family complete. |
| MSAA writeback remains a target/AA boundary. | Camera MSAA can request an AA mode and unsupported sample counts degrade through `AntiAliasFallbackReport`; no sorted-camera MSAA writeback blit exists. | Add multisampled target ownership, sorted-camera writeback policy, resolve/writeback graph node, and target-aware diagnostics before calling MSAA writeback complete. |
| Validation is focused and not full Bevy parity. | Existing graph tests cover disabled effects, missing resources, duplicate outputs, missing dependencies, and cycles. | Current-checkout M10W validation passed the focused post-process graph tests, AA fallback tests, pipeline/pass-order tests, and `cargo check -p zircon_runtime --lib --locked`; this still does not promote missing effect families. |

2026-05-26 M10W validation evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked render_product_post_process --jobs 1 --message-format short --color never`: PASS, 9 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_ui_graph_pass_order --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never`: PASS, 39 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers disabled-effect elision, missing scene color, invalid history input, duplicate output resources, missing effect dependency, dependency cycles, temporal slot split, and authored effect-stack ordering before final composite.

`zircon_runtime/src/graphics/tests/render_framework_bridge.rs` covers renderer-facing stats, verifies bloom, color grading, effect stack, and final composite are recorded as product postprocess nodes without appending synthetic entries to the normal render graph pass list, and checks that history resolve is recorded only after compatible frame history exists.

2026-06-02 render-main-chain focused validation added the temporal slot split regression: `cargo test -p zircon_runtime --lib --locked render_product_post_process --jobs 1 --message-format short --color never` passed 10 focused tests, and `cargo test -p zircon_runtime --lib --locked product_postprocess_executor_rejects_missing_gpu_resources --jobs 1 --message-format short --color never` passed the executor resource-binding regression. Both commands used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain` and emitted only pre-existing UI/accessibility/text warnings outside this postprocess lane.

2026-06-02 render-main-chain SSAO/cluster/bloom executor split validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib --locked executor_requires_post_process_context_instead_of_nooping --jobs 1 --message-format short --color never` passed 4 executor context regressions, `cargo test -p zircon_runtime --lib --locked graph_execution --jobs 1 --message-format short --color never` passed 32 graph execution tests, `cargo test -p zircon_runtime --lib --locked pipeline_compile --jobs 1 --message-format short --color never` passed 43 pipeline compile tests, `cargo test -p zircon_runtime --lib --locked render_product_post_process --jobs 1 --message-format short --color never` passed 10 post-process tests, and `cargo test -p zircon_runtime --lib --locked render_product_submit --jobs 1 --message-format short --color never` passed 11 submit tests. A later `render_framework_stats_report_executed_render_graph_passes` run was blocked by unrelated active plugin export-build-plan visibility errors in `zircon_runtime/src/plugin/export_build_plan/*`, not by this post-process lane.

2026-06-02 render-main-chain post-process graph record-owner validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib stage_execution_records_post_process_graph_through_record_owner --locked --jobs 1 --message-format short --color never` passed the graph-stage record owner regression, `cargo test -p zircon_runtime --lib render_product_post_process --locked --jobs 1 --message-format short --color never` passed 10 post-process tests, `cargo test -p zircon_runtime --lib render_framework_stats_report_executed_product_postprocess_nodes --locked --jobs 1 --message-format short --color never` passed the renderer stats regression, and `cargo test -p zircon_runtime --lib graph_execution --locked --jobs 1 --message-format short --color never` passed 35 graph execution tests.

2026-06-02 render-main-chain M7 volume-stack slice added `post_process/volume.rs` unit coverage for priority-ordered global blending, layer-filtered local blend influence, and extended effect-stack parameter resolution, plus `render_framework_stats_report_volume_effect_stack_product_node_when_authored` to prove a volume-authored effect stack resolves before product graph validation and records the `effect-stack` node. The follow-up extended-parameter slice added `effect_stack_settings.rs::tests::extended_effect_stack_settings_enable_product_node_without_legacy_fields` and `render_product_post_process_extended_effect_stack_settings_enable_product_node`, proving tonemap/dither/SSR-style settings can activate the product node without relying on the older vignette/grain/fog fields. The renderer consumption slice added `effect_stack_settings_are_encoded_into_post_process_params` plus `PostProcessParams` / `post_process.wgsl` fields so the concrete post-process shader consumes the resolved effect stack. The diagnostic-report slice added `RenderPostProcessEffectStackReport`, `last_post_process_effect_stack_report`, focused assertions for direct/volume-authored active families, approximation-backed families, missing authored LUT and SSR normal resources, and runtime diagnostics coverage for the `render.post_process.effect_stack.*` `DiagnosticStore` paths. The fallback resource slices added binding 10 fallback LUT sampling and binding 11 graph `scene-depth` sampling for DoF/SSR/fog baselines. The camera-depth slice added `PostProcessParams.effect_depth`, near/far/projection encoding coverage, and shader depth linearization for baseline DoF/SSR/fog. The authored-LUT binding slice added an enabled-LUT resource-preparation filter, a prepared texture-view lookup on `ResourceStreamer`, renderer selection between authored LUT and S-curve fallback at binding 10, and report coverage proving authored LUT is active but no longer approximation-backed. The LUT readiness/request slice added `last_post_process_lut_{request,ready,fallback}_count`, `render.post_process.lut.*` diagnostics, positive-intensity request semantics, and regressions for an enabled LUT without a texture handle. Focused validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 12 LUT-filtered tests, `cargo test -p zircon_runtime --lib effect_stack_report_records_active_approximated_and_missing_resources --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the missing-handle report regression, `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the diagnostics bridge regression, and focused `effect_stack_settings_are_encoded_into_post_process_params`, `orthographic_camera_depth_params_disable_perspective_linearization`, and `post_process_shader_samples_bound_scene_depth_texture` runs passed the renderer-param and scene-depth shader regressions. The first LUT run exposed stale test imports in `register_pipeline_asset` and `reload_pipeline`; those tests now import executor types from `crate::graphics` instead of the crate root, preserving the narrower runtime public surface.

2026-06-02 render-main-chain LUT shape-contract follow-up added `RenderColorLookupTextureLayout`, 2D-strip shape validation, explicit 3D LUT pending/fallback classification, prepared texture descriptor retention, and runtime diagnostics for `texture_2d_strip_ready_count`, `texture_3d_request_count`, and `unsupported_shape_count`. The renderer now inspects the texture asset's `RenderImageDescriptor` before preparing a post-process LUT, so invalid shapes and 3D LUTs do not create a WGPU D2 bind group accidentally. Focused validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 16 LUT-filtered tests, `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the diagnostics bridge regression, and `cargo test -p zircon_runtime --lib render_framework_stats_report_effect_stack_product_node_when_authored --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the RenderFramework effect-stack stats regression.

2026-06-02 render-main-chain 3D LUT binding follow-up moved explicit `Texture3d { size }` LUTs from pending/fallback-only status to a renderer-private WGPU binding path. `ResourceStreamer` now keeps a dedicated post-process LUT cache that uploads decoded `TexturePayload::Rgba8` assets using the retained `RenderImageDescriptor` dimension, final postprocess bind group binding 12 exposes `texture_3d`, and `post_process.wgsl` selects disabled/2D/2D-strip/3D sampling through `PostProcessParams.effect_flags.y`. The fallback set now includes both the 64x1 S-curve LUT and a 2x2x2 identity cube, so missing or failed authored textures remain deterministic. Focused validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 19 LUT-filtered tests, `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the diagnostics bridge regression, `cargo test -p zircon_runtime --lib render_framework_stats_report_effect_stack_product_node_when_authored --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the RenderFramework stats regression, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings. The basic text `.cube` import gap described after that slice is closed by the 2026-06-03 cube LUT asset ingress below.

2026-06-03 render-main-chain 3D LUT sampling follow-up added binding 13 for the renderer-owned clamp/linear LUT sampler and changed the 3D branch from nearest `textureLoad(...)` to texel-center `textureSampleLevel(...)`. This closes the renderer-side interpolation gap against Unity's bilinear `Texture3D` import behavior while keeping 2D-strip sampling on the existing atlas-safe `textureLoad(...)` path. Focused validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: `cargo test -p zircon_runtime --lib post_process_shader_samples_bound_effect_lut_texture_3d --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the shader contract regression, `cargo test -p zircon_runtime --lib lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 19 LUT-filtered tests, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings.

2026-06-03 render-main-chain cube LUT asset ingress added `zircon.builtin.texture.cube_lut` to `AssetImporter::default()`. The parser reads text `.cube` files, skips common metadata rows, rejects 1D shaper sections explicitly until a real shaper policy exists, enforces `LUT_3D_SIZE` in the shared `2..=256` range, requires exactly `size^3` RGB samples, clamps samples to RGBA8, and emits a linear D3 `TextureAsset` that the renderer can consume through the existing 3D LUT path. Focused validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`: before the explicit shaper guard, `cargo test -p zircon_runtime --lib cube_lut --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 5 cube-filtered tests, `cargo test -p zircon_runtime --lib texture_importer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 14 texture importer tests, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings. A Windows shaper-guard rerun was blocked before test execution when the target-dir dep-info fingerprint path disappeared during Cargo compile; the isolated WSL rerun `wsl -e sh -lc 'cd /mnt/e/Git/ZirconEngine && CARGO_TARGET_DIR=/tmp/zircon-render-main-chain-cube-lut-0603 cargo test -p zircon_runtime --lib cube_lut --locked --jobs 1 --message-format short --color never'` passed 6 cube-filtered tests, 2500 filtered out, with existing warnings only.

2026-06-03 render-main-chain SSR normal/depth-backend follow-up connected `gbuffer-normal` into the effect-stack graph and concrete postprocess bind group. `PostProcessStackDescriptor` now declares `gbuffer-normal` as an effect-stack input when SSR is active, `post.stack` passes the graph-owned normal view to binding 14, and the shader decodes that normal before choosing the SSR seed direction. The effect-stack report now receives renderer-resource status from the postprocess graph, so `effect-stack.ssr.normal` is not reported missing when `gbuffer-normal` is declared. The same slice added `PostProcessDepthSamplingMode`: non-GL backends keep the raw `texture_depth_2d` binding at binding 11, while GL/WebGL/ANGLE backends compile a fallback WGSL variant and bind the renderer-owned float fallback texture because WGPU/Naga's GLSL path cannot safely compile the raw depth sampling shape in this shader. Validation evidence: WSL `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed with existing warnings; Windows focused tests passed for `post_process_viewport_depth_fallback_shader_parses_for_gl_backends`, `viewport_depth_fallback_shader_removes_raw_depth_texture_sampling`, `effect_stack_resource_status_detects_graph_bound_ssr_normal`, `effect_stack_report_treats_bound_ssr_normal_as_available`, `effect_stack_ssr_declares_depth_and_normal_inputs`, and `render_product_post_process_extended_effect_stack_settings_enable_product_node`. WSL GPU bridge execution remains blocked in this environment: the un-suppressed lib-test compile hit a Rust 1.94.1 diagnostic ICE while emitting existing `dead_code` warnings, and the warning-suppressed GL run of `render_framework_stats_report_effect_stack_product_node_when_authored` produced a driver-level SIGSEGV in thread TLS teardown; `WGPU_BACKEND=vulkan` returned `NoAdapter`.
