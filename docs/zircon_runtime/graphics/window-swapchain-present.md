---
related_code:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_impl/trait_impl.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - user: 2026-05-10 switch runtime rendering to window swapchain present for RenderDoc capture
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - cargo check -p zircon_runtime
  - cargo check -p zircon_app
doc_type: module-detail
---

# Window Swapchain Present

## Ownership

`zircon_app` owns process entry, winit windows, raw native handles, and fallback presentation. It does not depend on wgpu and does not construct renderer objects. The app extracts a `ZrRuntimeNativeSurfaceTargetV1` from the active winit window, asks the loaded runtime whether the optional surface-present ABI is available, and then prefers `bind_viewport_surface` plus `present_viewport` on redraw. If the runtime lacks those fields, rejects the target, or present fails, the app disables surface present and falls back to `capture_frame` plus `SoftbufferRuntimePresenter`.

`zircon_runtime` owns the rendering surface lifecycle. The dynamic ABI validates viewport, ABI version, size, and native target data, maps the ABI DTO into `RenderViewportSurfaceDescriptor`, and delegates through `RuntimeRenderBridge` into the `RenderFramework` trait. The framework contract is backend-neutral: it exposes bind, unbind, and present-extract methods without leaking winit or raw-window-handle types to app code.

## Render Flow

The window path still renders the full scene through the existing extract, pipeline, runtime-prepare, history, feedback, overlay, and UI path. The difference is the final step. `present_frame_extract` renders into the renderer's offscreen final color target, then `ViewportSurface::present_texture` acquires a wgpu `SurfaceTexture`, draws a full-screen blit from the offscreen final color view into the swapchain texture, submits that GPU work, and calls `SurfaceTexture::present()`.

This first window-present slice deliberately keeps the offscreen render target as the product render target. That preserves existing post-process, history, UI composition, capture, and readback behavior while removing the CPU readback from the live window redraw path. It also gives RenderDoc a real present boundary, so captures should report the runtime as presenting instead of `D3D12 (Not Presenting)` when the window path is active.

## Surface Details

The initial native target implementation is Win32 because the current app-side raw handle extraction only emits Win32 handles. The runtime creates a wgpu unsafe raw-handle surface from `HWND` plus optional `HINSTANCE`, queries the existing adapter's surface capabilities, configures the surface with a preferred BGRA/RGBA format, FIFO-compatible present mode, and a frame latency of two. The present blit uses a small wgpu pipeline so the swapchain format does not need to match Zircon's `Rgba8UnormSrgb` offscreen format.

Surface loss, outdated swapchains, occlusion, and timeout are treated as non-fatal present skips. Validation errors and surface creation/configuration failures are returned to the dynamic API, causing the app to fall back to softbuffer readback presentation.

## Validation Notes

The automated coverage verifies that the dynamic runtime exports the optional surface-present functions, rejects wrong ABI and unknown viewport requests before session lookup, and that the app runtime loader only treats the window-present path as supported when all optional ABI fields are present and inside the reported table size. Manual acceptance still requires launching the runtime window under RenderDoc and confirming that RenderDoc reports a presenting API and that the timeline ends in a swapchain present after the `zircon-present-blit-pass`.
