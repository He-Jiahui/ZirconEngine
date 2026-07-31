# Shader06 M5 Direct-Present Performance Follow-up

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M5
Status: in_progress
Files: ["docs/zircon_runtime/graphics/window-swapchain-present.md", "docs/plans/zircon_runtime/shader/06/2026-07-29-m5-direct-present-performance.md", "docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_direct_present_20260729_dx12.md", "docs/tests/runtime/shader/zircon_shader_pbr_viewer_m5_direct_present_20260729_dx12_renderdoc_capture.rdc", "zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/frame_io.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/main.rs", "zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs", "zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs", "zircon_runtime/src/graphics/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_surface.rs", "zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/mod.rs"]

## Reason

Historical source measurements split the apparent 82--100 s HDRI load into cold/warm scene construction of about 95.54/87.60 s, with renderer initialization alone accounting for 90.55/86.48 s and an IBL cache restore only about 0.89 s. The current environment-only PBR profile has already removed the dominant DX12 deferred PSO startup cost: cache-reused scene construction is about 8.28 s and Ready is about 19.21 s. Split Ready-frame timing records about 9.87--9.99 s in the CPU wall-clock interval from renderer-frame entry through resource preparation, command encoding, and queue submission return, while `SceneRenderer::render` readback/completion is only 16--40 ms. That interval is not a GPU execution-duration measurement. The direct-surface form additionally includes surface acquisition, the GPU blit, and present before it returns, so its log names the interval `render_and_present_call`. The CPU RGBA readback followed by `SoftbufferViewportPresenter` upload is still an avoidable interactive-path round trip, but it is not the dominant cause of the long first frame. Direct present removes that round trip; GPU attribution remains a separate Render17 timestamp-query task.

Static tracing now identifies the actual first-frame CPU contributor for this viewer: `MeshPipelineCache::new` leaves its Base pipeline map empty, and `ensure_pipeline_for_variant` assembles Standard-PBR WGSL, creates a shader module, and creates a Base render pipeline for the first mesh variant. The environment-only viewer records `BaseScenePass`, not a GBuffer pass. That lazy mesh path is not included in the existing deferred `standard_pso` startup metric, so prewarming deferred lighting alone cannot eliminate it. The next capture must measure this Base-path hypothesis rather than treating it as accepted attribution.

The existing shader-prewarm workflow is not a runtime pipeline cache: it writes WGSL variants to disk and its WGPU pipeline mode creates disposable validation pipelines on a separate offscreen device. It therefore cannot populate this renderer's `MeshPipelineCache` or reuse the DX12 pipeline object that the first PBR frame creates. Any startup optimization must explicitly distinguish disk-source reuse from runtime/driver pipeline prewarming.

`wgpu` 29 does expose a persistent `PipelineCache`, but its documented backend support is currently Vulkan-only. The DX12 viewer therefore cannot use that API to persist the first Base PSO; the current Base descriptor's `cache: None` is not the missing Windows implementation. The practical next step is to measure and reduce the on-device lazy Base creation, or move that known work into an explicitly reported loading phase, rather than adding a cache that cannot serve the target backend.

The viewer must not prewarm by calling the CPU-image `SceneRenderer::render` and discarding its `ViewportFrame`: that would violate the contract which reserves this path for screenshots and image consumers. Shader06 now supplies `MeshPipelineCache::prewarm_environment_only_pbr_base_pipeline`, which resolves the current builtin PBR resource revision from that renderer's `ResourceStreamer` and creates the exact static, no-texture, no-shadow-receiver Standard-PBR Base pipeline in the renderer's own cache without encoding, submission, presentation, or readback. `EnvironmentOnlyPbrPreview` also defers its unused deferred-lighting foundation and PSO until a compiled-graph caller needs them; FullScene retains eager startup construction. Render17 invokes the Base prewarm after `ResourceStreamer` initialization and exposes its synchronous DX12 PSO cost in the startup report. A fresh direct-present run must still prove the first viewer frame reuses that entry; a Render17 generic no-readback offscreen warmup API remains the correct future route for scene-derived variants not covered by this known viewer pipeline.

## Scope

- Expose a `SceneViewportSurface` owned by `SceneRenderer`, created only after the native window exists.
- Render the existing final sRGB offscreen texture through the backend's GPU surface blit and present it without `ViewportFrame` construction or texture readback.
- Preserve the CPU `SceneRenderer::render` contract exclusively for screenshot/image consumers.
- Invoke the Shader06-owned exact-cache Base prewarm after `ResourceStreamer` initialization for the known environment-only Standard-PBR viewer variant; keep its synchronous cost explicitly reported as startup work.
- Use a Render17-owned no-readback offscreen warmup API only for scene-derived variants beyond that known cache key; do not call the CPU-image `SceneRenderer::render` only to discard its output.
- Keep the PBR viewer fallback deterministic: if direct presentation cannot acquire or validate the native surface, detach it and request one CPU-present redraw.
- Record direct-present timing only when a one-shot RenderDoc capture requests it. Its readback interval must remain zero, while the log retains the separate `render_and_present_call` CPU wall-clock interval for follow-up diagnosis. GPU duration requires a Render17 timestamp-query report.

## Completion Criteria

- Managed Windows exact test `cargo test -p zircon_runtime --lib --locked --features dynamic-api runtime_environment_only_pbr_base_prewarm_populates_the_renderer_cache -- --exact --test-threads=1` passes on the current source, proving that the same registry-revision viewer key creates one validated runtime Base pipeline and the second request reuses it.
- Managed Windows `cargo check -p zircon_app --bin zircon_shader_pbr_viewer --locked` and focused viewer tests pass on the current source.
- A fresh DX12 Debug viewer capture uses the direct surface path, logs `readback_and_completion=0ns`, retains the `render_and_present_call` CPU wall-clock interval, and exits after the one-shot capture. This is not an acceptance claim about GPU duration or that direct present eliminates GPU latency.
- Any pipeline-warmup evidence must either use the owner-wired same-cache Standard-PBR primitive or the owner-provided no-readback API, retain its cost within scene construction, and record the post-Ready direct-present timing separately; no result may characterize warmup as a reduction in total load work.
- A claimed first-frame miss reduction requires a fresh direct-present run to show that the already-wired primitive reuses the cache entry; the primitive alone is not that evidence.
- `D:\Tools\renderdoc\renderdoccmd.exe replay --loops 1` accepts the new capture.
- The resulting capture metadata and performance log are retained under `docs/tests/runtime/shader`; this record changes to `accepted` only after those results exist.

## Current Validation

The fresh managed `dynamic-api` exact gate completed as job `ba675041d1894317b88fe9c1a53a6987` / run `eb7aa86ba49c4ad390432129a685f00f`: `graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::ensure_pipeline::tests::runtime_environment_only_pbr_base_prewarm_populates_the_renderer_cache` ran once and passed. It proves the environment-only viewer key creates one same-cache Standard-PBR Base pipeline and a second request reuses it; the cold dynamic-api compilation took 31m34s and the actual test took 5.63s.

The remaining current-source gates are intentionally separate: the deferred-lighting on-demand test must show that the environment-only profile does not build the unused deferred PSO until required, and a fresh DX12 Debug viewer build/capture must exercise the direct surface path before its RenderDoc replay and screenshot metadata can replace historical artifacts. The former is coordinator-managed; no stale reservation or earlier executable may be reused. The prior viewer Debug build stopped in an external `frame_capture` parent-module import before viewer compilation, and the owner repair is now present in current source, so this gate requires a fresh build rather than a retry of its old result.

Implementation and source review are complete, but these outstanding managed results keep this record `in_progress`; they delay only acceptance and do not authorize a rollback or a duplicate failure record.

## Cross-Plan Coupling

`SceneViewportSurface` is a Render17-owned scene/framework projection. The open Render17 handoff `docs/plans/zircon_runtime/render/17/failure-2026-07-29-scene-viewport-surface-projection-drift.md` must retain the wrapper and complete its single-owner transfer into `ViewportRecord`; Shader06 consumes that result from the PBR viewer and does not add a second surface store or compatibility alias.

## Non-goals

- Screenshot export remains a CPU readback workflow and is not used to measure interactive presentation.
- This does not alter the accepted environment-only deferred-lighting profile, PMREM, SH9/IEM, probe selection, or M4 realtime-IBL work.
- The surface blit reuses the current backend format/present-mode policy; it does not introduce an independent swapchain implementation.
