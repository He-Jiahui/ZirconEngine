---
related_code:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_object_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_motion_vector_object_history.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_previous_motion_vector_object_history.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_object_history.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_motion_vector_object_history.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_previous_motion_vector_object_history.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
plan_sources:
  - user: 2026-06-02 implement ZirconEngine WGPU render main-chain closure plan
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_phase_sort_key_uses_unified_queue_layer_depth_order
  - zircon_runtime/src/core/framework/tests.rs::geometry_phase_inputs_feed_unified_sort_components_into_queue
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs::render_product_sprite_phase_queue_honors_material_queue_and_ui_z_index
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_carries_scene_camera_order_report_for_scene_camera
  - zircon_runtime/src/scene/tests/render_extract.rs::explicit_camera_render_frame_extract_has_no_scene_camera_order_report
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_stats_report_scene_camera_ordering_metadata
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_surface_offscreen_submit_and_capture_survive_unbind_noop
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_requires_render_target_usage
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_requires_renderable_render_target_format
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_render_target_metadata_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_ignores_non_texture_targets
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_waits_for_target_descriptor
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_accepts_matching_srgb_format
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_accepts_linear_rgba_target_for_conversion
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_marks_srgb_texture_ready_for_direct_import
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_keeps_linear_texture_on_conversion_writeback_path
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_blocks_unsupported_target_format
  - zircon_runtime/src/graphics/types/viewport_motion_vector_object_history.rs::tests::object_motion_history_keeps_only_dynamic_mesh_transforms
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs::tests::build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs::tests::direct_runtime_frame_submit_projects_resolved_output_target
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs::tests::output_target_texture_id_uses_resolved_texture_target_only
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs::tests::output_target_texture_id_ignores_non_texture_targets
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::output_target_graph_import_report_marks_srgb_texture_ready_for_direct_import
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::output_target_graph_import_report_keeps_linear_texture_on_writeback_path
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs::tests::import_frame_targets_rebinds_final_aliases_to_imported_texture_target
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs::tests::compiled_scene_outputs_can_carry_output_target_graph_import_report
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_srgb_target_imports_direct_graph_final_target
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_texture_direct_graph_import_execution
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_capture_source_report
  - zircon_runtime/src/core/framework/render/capture.rs::tests::captured_frame_new_defaults_to_primary_framework_offscreen_source
  - zircon_runtime/src/core/framework/render/capture.rs::tests::texture_capture_report_distinguishes_direct_import_and_conversion_sources
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::camera_target_writeback_report_separates_copy_and_conversion_debug_markers
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_executes_ready_copy_and_conversion_plans
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_report_maps_ready_and_blocked_plans
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_accepts_matching_source_and_destination
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_rejects_source_size_mismatch
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_rejects_destination_size_mismatch
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs::tests::output_target_texture_usages_prepare_render_target_only_without_sampled_binding
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs::tests::output_target_texture_usages_preserve_copy_and_sampled_authoring_flags
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs::renderdoc_debug_marker_registry_covers_capture_timeline
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors
  - cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked unified_sort_components --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_product_sprite_phase_queue_honors --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Render Frame Extract

## Purpose

`RenderFrameExtract` is the neutral frame DTO submitted through `RenderFramework`. Scene and runtime producers fill it directly so graphics code can compile and execute render graph work without reading editor state or concrete world internals.

## View Size Contract

`RenderViewExtract` records an optional `target_size` alongside the camera snapshot. The size is derived from an explicit viewport rectangle or headless camera target when the extract is created, and `RenderFrameExtract::apply_viewport_size(...)` updates both the camera aspect ratio and the stored target size before submission.

`RenderViewExtract::effective_view_size()` is the canonical read path for SRP and RenderGraph descriptor derivation. It clamps through the camera viewport when present and falls back to `1 x 1` only when the extract does not yet know a surface or headless target size.

During submit, `build_frame_submission_context(...)` resolves the camera target against the viewport record and, for `RenderCameraTarget::Texture`, the referenced `TextureAsset` metadata before cloning the extract with `apply_viewport_size(...)`. Valid texture targets must be nonzero 2D single-layer single-mip descriptors with `RenderImageUsage::RenderTarget` and a renderable RGBA8 format; their descriptor extent becomes the submission size just like a headless target's explicit size. `RenderCameraTargetResolutionReport` then records the target kind, primary viewport size, resolved target size, effective view size, and dynamic-resolution-scaled render size into `RenderStats` and `render.camera.target.*` diagnostics.

The same context also resolves a crate-internal `ViewportRenderOutputTarget` and attaches it to renderer-bound `ViewportRenderFrame` values. Generated extract submits and direct runtime-frame submits both carry the resolved target kind, headless size, or texture handle plus size after preflight. This keeps texture writeback and target-aware capture from revalidating authored camera state while `RenderFrameExtract` remains neutral and still carries no WGPU surface or texture object.

`ViewportRenderOutputTarget::writeback_plan(...)` is the renderer-internal planning seam. Non-texture targets report `NotRequested`; texture targets without a descriptor format report `PendingTargetDescriptor`; texture targets matching the framework offscreen output format label `rgba8unorm_srgb` report `ReadyForSrgbCopy`; linear `rgba8unorm` targets report `ReadyForConversion`; and unsupported target formats report `BlockedFormatMismatch`. The plan records texture handle, resolved size, source format, and target format without exposing resource-streamer texture internals.

`ResourceStreamer::ensure_scene_resources(...)` notices the resolved texture output target on `ViewportRenderFrame` and asks a dedicated output-target residency path to prepare that texture. Primary-surface and headless targets are ignored. Output-target residency uses `OutputTargetTextureResource` instead of the sampled material/sprite `GpuTextureResource`, so a valid camera target may be render-target-only without also needing sampled binding usage. When an sRGB target is prepared and the graph-import plan is ready, the renderer imports the prepared texture view as the final graph target aliases instead of binding those aliases to the framework offscreen final color. Linear `rgba8unorm` targets still use `ResourceStreamer::execute_output_target_writeback(...)` after graph execution: the fullscreen conversion pass samples the framework final color and writes the prepared linear target. The writeback report records skipped-direct-import, blocked, ready, copied, or converted status plus target extent, copy/conversion counts, and separate copy/conversion debug-marker emission. Prepared texture internals and WGPU handles stay behind the renderer resource owner instead of entering the frame DTO.

The renderer also records a separate graph-import report from that prepared output target. `ViewportRenderOutputTarget::graph_import_plan(...)` marks matching `rgba8unorm_srgb` texture targets as `ReadyForDirectImport` during residency/readiness preflight, linear `rgba8unorm` targets as `RequiresConversionWriteback`, unsupported formats as `BlockedFormatMismatch`, and non-texture targets as `NotRequested`. Graph execution upgrades successful sRGB imports to `DirectImported`, increments `direct_import_count`, and makes output-target writeback report `SkippedDirectImport`; readiness-only reports keep `direct_import_count` at zero. `RenderStats.last_camera_target_graph_import` and `render.camera.target.graph_import.*` expose those status/count/extent rows without moving WGPU texture handles into `RenderFrameExtract`.

`ViewportFrame` now carries `RenderCaptureReport` from the renderer readback path into `record_capture(...)`, and `CapturedFrame` stores the same report in the viewport record. `record_submission(...)` then forwards the stored report into `SubmissionRecordUpdate`, so `update_base_stats(...)` can publish `RenderStats.last_capture_report`. This is intentionally result metadata: it records whether the capture came from framework offscreen color, an imported texture target, a converted texture writeback, or a copied texture writeback, while `RenderFrameExtract` remains the authored-frame DTO and still carries no backend texture handle.

## Motion Vector History Handoff

The submit context also carries renderer-private temporal state that is derived from previous successful submissions rather than from editor state. `ViewportRecord` stores the previous camera snapshot separately from color-history validation, and it also stores `ViewportMotionVectorObjectHistory`, a dynamic-mesh-only map from entity id to prior transform. `build_frame_submission_context(...)` copies both histories into `FrameSubmissionContext`, and `build_runtime_frame(...)` attaches both to the renderer-bound `ViewportRenderFrame`.

This slice is a handoff contract only. It makes previous dynamic object transforms visible to later renderer passes through the frame DTO, but it does not yet add a mesh/skinned/particle velocity writer, widen mesh instance uniforms, or change the current camera/background `scene-motion-vector` producer.

## Scene Camera Scheduling Metadata

`RenderViewExtract` also carries optional scene camera provenance: `scene_camera_entity` and `scene_camera_order_report`. Scene-backed producers fill these fields after running render-extract systems so later multi-camera scheduling, diagnostics, and editor overlays can see the same active-camera ordering evidence as the scene. Synthetic extracts, snapshot adapters, and explicit `SceneViewportExtractRequest::camera` overrides leave the fields empty because they are not owned by a scene camera entity.

During WGPU submit the report is copied into `FrameSubmissionContext` and projected into `RenderStats.last_scene_camera_scheduled_count` plus `RenderStats.last_scene_camera_order_ambiguity_count`, then into `render.camera.scheduled_count` and `render.camera.order_ambiguity_count` diagnostics. The report is intentionally scheduling metadata only. WGPU submission still renders the single effective `view.camera`; split-screen execution and camera-stack submission remain separate renderer milestones.

## Sort Key Contract

`RenderPhaseSortKey` now exposes `RenderPhaseSortComponents` as the shared ordering input for 3D, 2D, UI, overlay, and debug draw records. The packed order is render queue, material queue, order in layer, UI z-index, depth or reverse depth for transparent phases, then entity tie-breaker.

`GeometryPhaseInput`, `SpritePhaseExtractInput`, `MeshPhaseInput`, and `SpritePhaseInput` carry the same queue fields with defaulting constructors. Meshes use depth plus entity tie-breaker by default; sprites map `z_order` to order in layer and can now add material queue, render queue, depth bias, and UI z-index without changing the queue builder contract.

## Design And Rationale

The size belongs on the extract, not in the SRP asset, because the same pipeline asset can be used for multiple viewports, headless targets, editor previews, and camera stacks. The compiler therefore receives the product pipeline and per-frame view data separately and derives graph resource descriptors from both.

This is still neutral data. No WGPU surface, texture, or swapchain object is stored in the framework DTO.

## Test Coverage

The focused pipeline compile test verifies that a headless HDR camera with 4x MSAA produces `scene-color` and `scene-depth` graph lifetimes with the expected extent, format, and sample count. Broader scene extract and renderer execution validation remains part of the milestone testing stage.

Focused validation on 2026-06-02 passed for `pipeline_compile` with 42 tests, plus the two direct phase-order filters for mesh unified sort components and sprite material queue/UI z-index ordering. These runs used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain` and emitted only pre-existing warning classes outside this change.
