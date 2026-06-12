---
related_code:
  - dev/bevy/crates/bevy_core_pipeline/src/lib.rs
  - dev/bevy/crates/bevy_core_pipeline/src/schedule.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom/execute_bloom.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/record_overlays.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/attachment_ops.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_preview_sky.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_bloom/execute_bloom.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/record_overlays.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
plan_sources:
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-05-21 continue M10 schedule visibility acceptance checklist
  - user: 2026-05-21 continue Bevy render schedule and submit pipeline evidence mapping
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs::tests::packed_sort_key_clusters_opaque_by_pipeline_before_tie_breaker
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs::tests::packed_sort_key_keeps_transparent_depth_before_pipeline
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs::tests::packed_sort_key_ignores_transparent_pipeline_variant
  - zircon_runtime/src/core/framework/tests.rs::render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d
  - zircon_runtime/src/core/framework/tests.rs::render_product_pipeline_camera_projection_selects_core_pipeline_kind
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_forward_plus_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_deferred_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs::metadata_context_exposes_attachment_ops_for_written_resource
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::preview_sky_executor_requires_preview_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::overlay_executor_requires_overlay_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::sprite_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::mesh_executor_requires_mesh_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::deferred_gbuffer_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::deferred_lighting_executor_requires_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::plugin_render_feature_descriptors_require_explicit_executor_registration
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs::particle_plugin_executor_ids_require_explicit_registration
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs::register_pipeline_asset_rejects_plugin_executor_from_descriptor_only
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs::register_pipeline_asset_accepts_plugin_executor_from_explicit_registration
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs::reload_pipeline_rejects_plugin_executor_from_descriptor_only
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs::reload_pipeline_accepts_plugin_executor_from_explicit_registration
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs::sprite_subpasses_apply_graph_attachment_ops_only_to_outer_draws
  - zircon_runtime/src/graphics/tests/project_render.rs::deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Core Pipeline Contracts

## Purpose

`zircon_runtime::core::framework::render::core_pipeline` owns the neutral camera-selected pipeline and render-phase vocabulary. Bevy's `bevy_core_pipeline` is explicitly camera driven: each camera runs a Core2d or Core3d schedule, and those schedules own the default 2D/3D phase families. Zircon maps that idea into DTOs and phase queues instead of copying Bevy's ECS schedules.

Concrete graph passes, WGPU command encoding, render pass assets, and resource preparation stay under `zircon_runtime::graphics`. The framework module names the product phases that scene extraction, sprite extraction, mesh extraction, and pipeline compilation agree on.

## Product Surface

`CorePipelineKind` selects `Core2d` or `Core3d`. `ViewportCameraSnapshot::core_pipeline_kind()` maps orthographic cameras to `Core2d` and perspective cameras to `Core3d`, giving product render extraction a Bevy-style camera-driven default without creating a second renderer.

`RenderPhase` names the shared phase family: 2D opaque, 2D alpha-mask, 2D transparent, 3D opaque, 3D alpha-mask, 3D transparent, prepass, shadow, deferred, post-process, UI, overlay, and debug.

`RenderPhaseItem` is the neutral queue row. It records the entity, phase, sort key, and whether the phase item came from a mesh or sprite source. The source distinction is important because M6A proved sprites must not be confused with particle billboards, and future Mesh2d rendering must not reuse sprite acceptance accidentally.

`RenderPhaseQueue` stores sorted phase items and exposes `items_for_phase(...)` for renderer or diagnostics consumers. `build_mesh_phase_queue(...)` and `build_sprite_phase_queue(...)` classify alpha modes into opaque, alpha-mask, or transparent phases for the selected core pipeline.

`RenderPhaseSortKey` keeps deterministic ordering local to the framework contract. Meshes sort by phase, depth, and entity tie-breaker; sprites sort by z order before depth and entity. Transparent phases reverse depth ordering inside that rule.

`packed_sort_key_u64(...)` is the graphics-facing bridge for the MD-M1 mesh command layer. It preserves the same queue-prefix inputs but emits the compact `u64` command sort key used by `MeshDrawCommand`. Non-transparent phases include a state bucket from pipeline variant plus material discriminant before the coarse depth/tie-breaker lane, so opaque/prepass/velocity command streams can cluster state. Transparent phases ignore pipeline/material state and keep ordered depth before the tie-breaker, preserving back-to-front semantics for alpha blending. The bit layout is still the MD-M1 transitional layout; plan 09 remains the authority for the final shared layout.

## Graphics Integration

`RenderPipelineAsset::default_core2d()` maps the neutral Core2d phases into concrete stages: `Opaque2d`, `AlphaMask2d`, `Transparent2d`, `PostProcess`, `Ui`, `Overlay`, and `Debug`. It enables the built-in sprite, post-process, UI, and debug overlay features.

`RenderPipelineAsset::default_forward_plus()` and `default_deferred()` map Core3d into concrete 3D render pass orders, including prepass/shadow/deferred or lighting stages before post-process, UI, overlay, and debug.

This two-layer design is intentional: the framework contract says what product phase a renderable belongs to, while `zircon_runtime::graphics` decides which graph pass and executor actually draw it.

## Bevy Render Schedule Evidence

Bevy's render foundation has three separate layers that Zircon must not flatten together. `dev/bevy/crates/bevy_render/src/lib.rs:120-128` states that rendering runs in a `RenderApp` sub-app which exchanges data with the main app between main schedule iterations, and may run between main iterations or in parallel when `PipelinedRenderingPlugin` is enabled. `lib.rs:151-208` names the default render schedule sets: extract command application, prepare assets, prepare meshes, create views, specialize, prepare views, queue, queue meshes, queue sweep, phase sort, prepare resources, prepare bind groups, render, cleanup, and post-cleanup.

Bevy's pipelined mode is explicitly separate from the normal render schedule. `dev/bevy/crates/bevy_render/src/pipelined_rendering.rs:68-105` documents the render thread model: sync and extract happen on the main thread, extract commands are applied on the render thread, the render schedule runs there, `RenderExtractApp` can run before I/O, winit events and the main app schedule run in parallel with rendering, and extraction waits for both sides to finish before starting the next frame. `pipelined_rendering.rs:111-122` inserts `RenderExtractApp` only if `RenderApp` exists, and `pipelined_rendering.rs:124-178` moves the render sub-app over bounded channels to the render thread.

Bevy's core pipeline is then camera-schedule driven inside that render app. `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:1-11` describes Core2d/Core3d schedules as per-camera sub-schedules. `schedule.rs:29-65` defines `Core3d` with `Prepass`, `MainPass`, `EarlyPostProcess`, and `PostProcess`; `schedule.rs:68-104` defines the same staged shape for `Core2d`. The camera driver at `schedule.rs:111-170` iterates sorted cameras, skips invalid window targets, inserts `CurrentView`, runs each camera's schedule, and records which windows were covered.

## Zircon Submit Schedule State

Zircon currently implements a synchronous submit pipeline, not a Bevy render sub-app. `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs:22-123` owns the frame submission sequence: build the submission context, lock framework state, begin optional debugger capture, prepare runtime plugin sidebands, resolve frame history, build the runtime frame, render through the selected compiled pipeline, collect runtime feedback, record the submission, release old history, and update stats.

Context building and preparation are explicit but not ECS schedule sets. `build_frame_submission_context/build.rs:19-150` resolves viewport state, target size, effective pipeline, enabled render features, effective post-process and anti-alias settings, visibility context, and frame-history validation. `prepare_runtime_submission/prepare.rs:8-31` prepares advanced runtime sidebands for Hybrid GI and Virtual Geometry; this is a narrow runtime preparation phase, not a general Bevy `PrepareAssets` / `PrepareResources` stage.

Concrete command encoding is still owned by the compiled-scene renderer. `render/render.rs` keeps depth-prepass, shadow, and ambient-occlusion in the unconditional early graph family; the depth-prepass stage now receives preview-sky overlay context plus `NormalPrepassPipeline` and mesh draw-list context so `sky.preview-*`, `mesh.depth-prepass`, and `deferred.depth-prepass` execute through the graph instead of private SceneRenderer calls. Forward+ lighting runs before the main forward scene path, while Deferred gbuffer and lighting execute inside `render_scene_passes(...)` after the graph-owned background/depth initialization. The same compiled-scene flow then runs the PostProcess stage through graph executor dispatch, copies history, and runs late UI/overlay/debug graph stages before submitting the command buffer and returning graph execution records. `execute_graph_stage.rs` iterates compiled pass stages for a `RenderPassStage`, validates each pass, inserts stage markers, builds a GPU execution context, dispatches the registered executor, and records pass execution.

Render pass execution contexts now expose graph attachment operations through `RenderPassExecutionContext::attachment_ops_for_write(...)`. This is the execution-side bridge for the render-main-chain M2/M3 attachment metadata: SRP compilation decides whether a texture write should clear or load, RenderGraph preserves that metadata on `RenderGraphPassResourceAccess`, and SceneRenderer executors can query the write target directly before translating the decision into WGPU load/store operations. The helper intentionally returns operations only for write accesses on transient texture or external target resources.

The concrete consumers now include preview sky, depth/normal prepass, `ui.screen-space`, sprite, forward mesh, Deferred gbuffer/lighting, `post.stack`, and `overlay.gizmo` executors. Preview-sky executors read the color/depth write operations, open the WGPU background pass through `ViewportOverlayRenderer::record_preview_sky_with_attachment_ops(...)`, and initialize the background before geometry. The prepass executor then reads `scene-depth` and `gbuffer-normal` write operations and maps them to WGPU depth/color load-store operations before calling `NormalPrepassPipeline`. `post.stack` requires the post-process stack context before dispatching SSAO, clustered lighting, bloom, and final post-process recording from the graph stage, replacing the former fixed `execute_post_process_stack(...)` wrapper. `ui.screen-space` reads the `viewport-output` write operation from the pass context and passes the neutral `RenderGraphAttachmentOps` into `ScreenSpaceUiRenderer::record(...)`. `overlay.gizmo` requires the graph-bound `viewport-output` and `scene-depth` resources plus prepared overlay buffers before it dispatches `ViewportOverlayRenderer::record_overlays(...)`; compiled-scene submission no longer calls that overlay draw path after graph execution. Sprite executors read the `scene-color` write operation, require `scene-depth`, and dispatch `SpriteRenderer::record(...)` through graph stage execution instead of a private scene-renderer bypass. Forward mesh executors split prepared mesh draws into opaque, alpha-mask, and transparent buckets and dispatch `BaseScenePass` through graph stage execution. Deferred executors route `deferred.gbuffer` and `lighting.deferred` through `DeferredSceneResources`, import `gbuffer-albedo` / `gbuffer-normal` from the offscreen target set, and dispatch post-lighting transparent meshes through the existing `mesh.transparent` executor. The legacy fixed `render_scene(...)` path still calls UI with explicit `Load + Store`, preserving existing behavior while graph-owned preview sky, UI, overlay, sprite, mesh, prepass, Deferred, and post-process paths move off private hard-coded rules.

Pipeline compilation and executor registration form Zircon's closest current analogue to Bevy's queue/render split. `render_pipeline_asset/compile.rs:18-90` validates core-pipeline compatibility, renderer assets, stage-to-phase mapping, feature descriptors, required extract sections, capability requirements, and history bindings. `compile.rs:111-180` builds graph passes, stage mappings, executors, queues, dependencies, and resources. `render_pass_executor_registry.rs` registers concrete built-in executors plus a narrow allow-list of product no-op executors, while descriptor-provided plugin executor ids must arrive through explicit plugin executor registrations. Descriptor-aware registry constructors are crate-internal so external plugin code cannot use the registry as an executor-admission shortcut. Compiled pipeline validation rejects missing executors before activation, and `register_pipeline_asset(...)` / `reload_pipeline(...)` now preserve that rule at runtime-framework entry points so linked feature descriptors cannot silently turn plugin passes into runtime-owned no-ops.

## M10J Completion Boundary

| Bevy render schedule area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Render app / render world | Zircon has a runtime render framework with explicit submit context and renderer state locks. It does not have a separate render world, render sub-app, or main/render app data-exchange boundary. | Keep the current runtime framework unless a real parallel render world is needed, but document the divergence and expose enough stage diagnostics that product users can reason about extract/prepare/queue/render phases. |
| Extract / prepare / queue / render sets | Zircon has frame extract DTOs, runtime preparation, graph compilation, graph executor dispatch, and submit stats, but not Bevy's named `RenderSystems` sets. | Add neutral schedule/stage names to diagnostics and acceptance docs before claiming Bevy-like render-stage parity. Avoid forcing ECS schedule semantics into non-ECS runtime internals. |
| Camera schedule execution | Zircon maps camera projection to Core2d/Core3d and orders cameras, but the concrete renderer is still mostly single active-view submit. | Add true multi-camera schedule execution, per-target coverage tracking, split-screen / render-to-texture routing, and uncovered-surface clearing before claiming camera-driven schedule parity. |
| Pipelined rendering | Zircon submit is synchronous from the caller's perspective. It has scoped profiling markers and RenderDoc markers, but no render thread, `RenderExtractApp`, or overlap telemetry. | Keep pipelined rendering as a separate scheduling milestone; do not conflate current synchronous submit stats with Bevy's frame-overlap model. |
| Graph executors | Zircon validates compiled graph executors and executes stage-declared passes through a registry. UI, overlay, sprite, mesh, prepass, Deferred, and post-process executors now fail on missing execution context instead of silently no-oping; plugin descriptor executor ids also require explicit executor registrations. | Extend executor diagnostics with queue choice, culled pass reason, resource residency, pass timing, and backend queue/capability status so renderer behavior is inspectable without Bevy's render world. |

## M10Q Schedule Visibility Acceptance Checklist

M10.2 is the schedule visibility gate. It does not require Zircon to copy Bevy's render sub-app, render world, or pipelined render thread before the default renderer can continue. It does require the synchronous path to expose enough neutral stages and graph decisions that later 2D, 3D, UI, presentation, post-process, AA, diagnostics, and advanced-render slices can identify where a failure occurred.

| Check | Bevy pressure | Zircon current evidence | Promotion requirement |
| --- | --- | --- | --- |
| Render ownership is named before execution. | `dev/bevy/crates/bevy_render/src/lib.rs:120-128` makes rendering a `RenderApp` sub-app, and `lib.rs:344-423` sets up the render sub-app, schedules, startup, recovery, pipeline cache, and render system. | `submit_frame_extract(...)` is a single runtime-framework entry point with scoped context-build, prepare, render, feedback, record, release, and stats steps (`submit/submit.rs:22-123`). | Keep the divergence explicit: M10.2 can promote schedule visibility without claiming render world or sub-app parity. |
| Stage names are observable. | Bevy names `RenderSystems` sets for extract commands, prepare assets, prepare meshes, create views, specialize, queue, phase sort, prepare, bind groups, render, cleanup, and post-cleanup (`bevy_render/src/lib.rs:151-208`), then chains the base schedule (`lib.rs:286-317`). | Zircon already has source-level phases: build submission context, prepare runtime submission, compile pipeline, execute graph stage, post-process, submit, record, update stats. | Add/maintain diagnostics that use stable neutral names for extract/context-build/prepare/queue-or-phase/graph-compile/render/postprocess/present/cleanup. |
| Camera-driven Core2d/Core3d stays separate from global submit. | Bevy defines Core3d and Core2d schedules as per-camera sub-schedules (`bevy_core_pipeline/src/schedule.rs:31-104`) and `camera_driver` iterates sorted cameras, skips invalid targets, inserts `CurrentView`, runs each camera schedule, and tracks covered windows (`schedule.rs:111-170`). | `ViewportCameraSnapshot::core_pipeline_kind()` maps camera projection to Core2d/Core3d; phase queue and pipeline compile tests cover ordering and Core2d/Core3d compatibility. | Multi-camera execution, per-target coverage, uncovered-surface clearing, split-screen, texture-target scheduling, and editor/runtime multi-view routing remain explicit M10.7/M10.2 follow-ups. |
| Graph executor choices are inspectable. | Bevy's queue/phase-sort/prepare/render sets make phase and backend transitions visible. | Zircon `compile.rs:18-180` validates core-pipeline compatibility, enabled feature descriptors, required extract sections, capability requirements, history bindings, stage mappings, queues, dependencies, graph resources, and attachment ops; `execute_graph_stage.rs:80-180` executes pass stages through registered executors and records executor id, stage, queue, declared queue, dependencies, and resources. `RenderPassExecutionContext::attachment_ops_for_write(...)` lets executors consume clear/load/store decisions without pass-name rules, and `ui.screen-space`, `overlay.gizmo`, `post.stack`, sprite, and forward mesh executors now apply or require graph-owned resources instead of private render-target access. | Add culled-pass reason, resource-residency decision, executor availability, effective queue, and pass timing to diagnostics before calling graph scheduling Bevy-complete. |
| Pipelined rendering remains a separate milestone. | Bevy `PipelinedRenderingPlugin` moves rendering to another thread so frame N rendering can overlap frame N+1 simulation (`pipelined_rendering.rs:68-105`) and only installs `RenderExtractApp` when `RenderApp` exists (`pipelined_rendering.rs:111-122`). | Zircon's current submit path is synchronous and uses profiling scopes plus RenderDoc markers, not render-thread channels or overlap telemetry. | Do not use synchronous submit stats as pipelined rendering proof; future work needs render-thread lifecycle, extraction handoff, overlap timing, and shutdown/drop-thread ownership evidence. |
| Validation is stage-scoped. | Bevy schedule sets let failures be localized by set. | Current tests cover phase queue ordering, camera projection selecting Core2d/Core3d, default Core2d/forward/deferred pipeline compile order, and core-pipeline mismatch errors. | M10.2 promotion must run focused core-pipeline/submit tests and `cargo check -p zircon_runtime --lib --locked`, or explicitly remain docs-only. |

This checklist records the source-backed gate and current evidence. It does not claim fresh Cargo validation for this checkout.

## Current Limits

This module is not a Bevy `RenderApp` or render graph scheduler. It does not run sub-app schedules, submit command buffers, clear uncovered swapchains, or allocate per-view targets.

The current extraction path is still mostly single-camera. Camera ordering and target routing are now explicit contracts, but true multi-camera Core2d/Core3d schedule execution, split-screen, render-to-texture scheduling, and editor/runtime multi-view routing remain later milestones.

## Test Coverage

`render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d` proves mesh alpha modes classify into the expected 2D and 3D phase order.

`render_product_pipeline_camera_projection_selects_core_pipeline_kind` proves the camera contract chooses Core2d for orthographic projection and Core3d for perspective projection.

The pipeline compile tests prove the default Core2d, forward-plus, and deferred pipeline assets map the neutral phases into concrete render pass stage order and required extract sections.

2026-06-02 render-main-chain validation used `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`. `cargo test -p zircon_runtime --lib --locked graph_execution --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 34 graph-execution tests, including the execution-context attachment-op bridge, preview-sky executor registration and missing-context rejection, the depth-prepass executor, the post-process stack executor rejecting missing post-process context, the screen-space UI executor consuming `viewport-output` attachment ops, the `overlay.gizmo` executor rejecting missing overlay context, and sprite/mesh/Deferred executors rejecting missing renderer context instead of no-oping. `cargo test -p zircon_runtime --lib --locked pipeline_compile --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 43 SRP compile tests. `cargo test -p zircon_runtime --lib --locked render_product_post_process --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 10 post-process product tests, `cargo test -p zircon_runtime --lib --locked render_product_submit --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed 11 submit tests, and `cargo test -p zircon_runtime --lib --locked render_framework_stats_report_executed_render_graph_passes --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed the graph execution stats regression with `preview-sky` followed by `depth-prepass`. These runs emitted only pre-existing UI/accessibility/text warnings outside this core-pipeline lane.
