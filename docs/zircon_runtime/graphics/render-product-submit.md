---
related_code:
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/config.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_asset.rs
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/runtime/history/is_compatible.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
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
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/viewport_generation_guard.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/generation.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/config.rs
  - zircon_runtime/src/graphics/backend/render_backend/graphics_debugger_capture.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/extract/history.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
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
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/generation.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_capture.rs
plan_sources:
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs
doc_type: module-detail
---

# Render Product Submit

M4A makes `RenderFrameExtract` the product submit authority. `ViewportRenderFrame::from_extract` keeps a legacy scene snapshot field for compatibility with older public-runtime paths, but renderer internals read camera, mesh, light, overlay, and preview data through accessors backed by `RenderFrameExtract`.

Core pipeline selection is neutral framework data. Cameras select `CorePipelineKind::Core2d` for orthographic projections and `CorePipelineKind::Core3d` for perspective projections. Unset viewport submit uses that extract-owned pipeline kind to choose the built-in Core2d or Forward+ Core3d pipeline, while explicit viewport pipelines and quality-profile overrides remain authoritative and are rejected at compile time if their `core_pipeline` does not match the submitted extract.

`HistoryResolve` is no longer part of the default effective pipeline. `RenderFeatureQualitySettings::default()` leaves `history_resolve` disabled, `BuiltinRenderFeature::HistoryResolve` requires explicit opt-in, and profile compilation enables it only when a profile calls `with_history_resolve(true)`. This keeps default Core3d rendering free of scene-color temporal blending until motion vectors, reprojection, camera-cut detection, and disocclusion checks exist.

`GeometryExtract` carries phase queues derived from material alpha mode plus the selected pipeline. Production world extraction reads the alpha hint stored on each `MeshRenderer` and creates phase inputs from the sorted mesh rows, so mesh draw construction can consume aligned opaque, alpha-mask, and transparent queues instead of falling back to raw mesh-vector order.

Pipeline compile validates that declared renderer stages with product phases have matching `RenderPipelineAsset.phase_mapping` entries. The enforced stage-to-phase mapping covers 2D mesh stages, 3D mesh stages, depth prepass, shadow, deferred, postprocess, UI, overlay, and debug; lighting and ambient occlusion remain product-phase-neutral until a dedicated phase exists. Runtime graph execution now calls the declared Core2d transparent, UI, and overlay stages through `execute_graph_stage` while retaining the concrete overlay and screen-space UI renderer calls.

Submit safety is guarded by viewport generations. Context building captures the viewport record generation while resolving size, effective pipeline, quality profile, and history state. Before runtime prepare mutates viewport runtime state, and again before recording the rendered frame back into the viewport, submit revalidates that the viewport still exists and that its generation matches. Missing viewports return `RenderFrameworkError::UnknownViewport`; changed viewports return `RenderFrameworkError::ViewportChanged` instead of relying on checked-then-`expect` panics.

Frame history reuse now includes an extract validation key. `build_frame_submission_context(...)` records world id, camera snapshot, mesh identity/transform/model/material/tint/mobility/layer mask, lighting extract, animation pose extract, post-process settings, particle extract, and the compiled effective feature names. `resolve_history_handle(...)` reuses the previous `FrameHistoryHandle` only when size, pipeline, history bindings, and this validation key all match. Camera motion, mesh motion, material/tint/layer changes, light changes, pose changes, bloom/color-grading/preview changes, particle changes, world changes, and feature toggles therefore allocate a new history handle before renderer history textures are reused. Renderer history copy also preserves slot semantics: `FrameHistorySlot::SceneColor` is copied from `OffscreenTarget.scene_color`, not from post-processed `final_color`, so bloom/color grading and later overlay/UI composition do not feed back as scene-color history.

Graphics debugger capture is a submit-scoped request, not a persistent rendering mode. The only live triggers are `RenderFramework::request_graphics_debugger_capture(viewport)` and `ZR_RENDERDOC_CAPTURE_NEXT=1`; editor UI and dynamic API commands do not currently expose a separate RenderDoc button. The trait method stores a pending viewport in `WgpuRenderFramework`; non-matching viewport submits leave it pending. The environment variable arms the first viewport created by the framework so desktop debug launches can capture the first rendered frame without editor code calling the trait method. On the matching submit, capture begins before runtime prepare/render command recording and finishes after the frame is produced. The blocking wgpu stop/poll step runs after the framework state mutex is released, while an operation lock remains held so no second frame or viewport/pipeline mutation can enter the active capture window. Destroying a viewport with pending or queued debugger capture clears that debugger state and records a destroyed-viewport error instead of leaving `capture_pending` true forever. The status query reports wgpu capture-hook availability, the selected wgpu backend as `wgpu(dx12)` / `wgpu(vulkan)` / equivalent, pending/active flags, the last captured frame generation, and any submit or stop error. `available` means the backend exposes the wgpu debugger capture hook; it does not prove RenderDoc is attached. If the matching submit fails during preflight before capture starts, the pending request is consumed and `last_error` records the preflight error. If a submit fails while capture is active, cleanup still stops the capture and clears active/pending state before returning the original error.

RenderDoc-readable markers are centralized in `zircon_runtime/src/graphics/debug_markers.rs`. The compiled-scene command encoder emits markers for `FrameExtract`, `Clear`, `Prepass`, `MainScene`, `Lighting`, `DeferredLighting`, `PostProcess`, `HistoryCopy`, `Overlay`, and `UI`; the readback path emits `Readback` before the GPU-to-CPU copy. Graph-stage execution maps `RenderPassStage::Lighting` to the generic `zircon::Lighting` marker so Forward+ lighting stages are not mislabeled as deferred, while the fixed deferred lighting pass still uses `zircon::DeferredLighting`.

For Windows RenderDoc capture, launch Zircon from RenderDoc with environment variables set before process start: use `WGPU_BACKEND=dx12` for Direct3D 12 or `WGPU_BACKEND=vulkan` for Vulkan, set `WGPU_DEBUG=1` and `WGPU_VALIDATION=1` when validation output is needed, and set `ZR_RENDERDOC_CAPTURE_NEXT=1` to capture the first created viewport's next submit. After capture, inspect the event browser for `zircon::FrameExtract`, `zircon::MainScene`, `zircon::Lighting`, `zircon::PostProcess`, `zircon::HistoryCopy`, and `zircon::UI`; history textures are labeled `zircon-history-scene-color`, `zircon-history-global-illumination`, and `zircon-history-ambient-occlusion`. The CPU fallback path still marks final readback as `zircon::Readback`, while the app-host window path now binds a native surface and finishes redraw through a wgpu swapchain `SurfaceTexture::present()` after a `zircon-present-blit-pass`. Keep `HistoryResolve` explicitly disabled unless the test scenario intentionally opts into temporal scene-color blending.

Validation coverage lives in `render_product_pipeline`, `render_product_submit`, `pipeline_compile`, `project_render`, `render_framework_bridge`, and `render_debugger_and_history` tests. The submit test intentionally verifies that direct extract frames can diverge from the legacy scene snapshot, proving product rendering must not use `to_scene_snapshot()` as the draw authority. The render debugger/history tests cover idle debugger status, exact DX12/Vulkan backend status under `WGPU_BACKEND`, backend env parsing, first-created-viewport capture arming, marker registry coverage, matching-viewport request consumption, unknown viewport rejection, destroyed pending-capture cleanup, history validation-key invalidation, and explicit history-resolve opt-in. Manual `.rdc` acceptance remains a desktop RenderDoc step because this automated gate cannot launch the GUI capture workflow.
