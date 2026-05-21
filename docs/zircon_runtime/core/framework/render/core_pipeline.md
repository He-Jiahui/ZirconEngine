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
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
plan_sources:
  - user: 2026-05-21 continue Bevy render schedule and submit pipeline evidence mapping
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d
  - zircon_runtime/src/core/framework/tests.rs::render_product_pipeline_camera_projection_selects_core_pipeline_kind
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_forward_plus_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_deferred_pipeline_compiles_expected_stage_order_and_passes
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

Concrete command encoding is still owned by the compiled-scene renderer. `render/render.rs:31-47` fixes the early and late stage families that are executed around the main scene path. `render/render.rs:64-123` validates the compiled pipeline, writes scene uniforms, creates the command encoder, imports graph targets, and starts executing early graph stages. `render/render.rs:136-203` runs the concrete scene passes, post-process graph, history copy, and late graph stages; `render/render.rs:217-233` records overlays, submits the command buffer, and returns graph execution records. `execute_graph_stage.rs:80-113` iterates compiled pass stages for a `RenderPassStage`, while `execute_graph_stage.rs:128-180` validates each pass, inserts stage markers, builds a GPU execution context, dispatches the registered executor, and records pass execution.

Pipeline compilation and executor registration form Zircon's closest current analogue to Bevy's queue/render split. `render_pipeline_asset/compile.rs:18-90` validates core-pipeline compatibility, renderer assets, stage-to-phase mapping, feature descriptors, required extract sections, capability requirements, and history bindings. `compile.rs:111-180` builds graph passes, stage mappings, executors, queues, dependencies, and resources. `render_pass_executor_registry.rs:19-53` registers built-in executors for post-process, sprite, UI, history, and no-op graph passes, while `render_pass_executor_registry.rs:123-140` rejects compiled pipelines that reference missing executors.

## M10J Completion Boundary

| Bevy render schedule area | Zircon product state | Completion requirement |
| --- | --- | --- |
| Render app / render world | Zircon has a runtime render framework with explicit submit context and renderer state locks. It does not have a separate render world, render sub-app, or main/render app data-exchange boundary. | Keep the current runtime framework unless a real parallel render world is needed, but document the divergence and expose enough stage diagnostics that product users can reason about extract/prepare/queue/render phases. |
| Extract / prepare / queue / render sets | Zircon has frame extract DTOs, runtime preparation, graph compilation, graph executor dispatch, and submit stats, but not Bevy's named `RenderSystems` sets. | Add neutral schedule/stage names to diagnostics and acceptance docs before claiming Bevy-like render-stage parity. Avoid forcing ECS schedule semantics into non-ECS runtime internals. |
| Camera schedule execution | Zircon maps camera projection to Core2d/Core3d and orders cameras, but the concrete renderer is still mostly single active-view submit. | Add true multi-camera schedule execution, per-target coverage tracking, split-screen / render-to-texture routing, and uncovered-surface clearing before claiming camera-driven schedule parity. |
| Pipelined rendering | Zircon submit is synchronous from the caller's perspective. It has scoped profiling markers and RenderDoc markers, but no render thread, `RenderExtractApp`, or overlap telemetry. | Keep pipelined rendering as a separate scheduling milestone; do not conflate current synchronous submit stats with Bevy's frame-overlap model. |
| Graph executors | Zircon validates compiled graph executors and executes stage-declared passes through a registry. | Extend executor diagnostics with queue choice, culled pass reason, resource residency, pass timing, and backend queue/capability status so renderer behavior is inspectable without Bevy's render world. |

## Current Limits

This module is not a Bevy `RenderApp` or render graph scheduler. It does not run sub-app schedules, submit command buffers, clear uncovered swapchains, or allocate per-view targets.

The current extraction path is still mostly single-camera. Camera ordering and target routing are now explicit contracts, but true multi-camera Core2d/Core3d schedule execution, split-screen, render-to-texture scheduling, and editor/runtime multi-view routing remain later milestones.

## Test Coverage

`render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d` proves mesh alpha modes classify into the expected 2D and 3D phase order.

`render_product_pipeline_camera_projection_selects_core_pipeline_kind` proves the camera contract chooses Core2d for orthographic projection and Core3d for perspective projection.

The pipeline compile tests prove the default Core2d, forward-plus, and deferred pipeline assets map the neutral phases into concrete render pass stage order and required extract sections.
