---
related_code:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_impl/trait_impl.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_present.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - user: 2026-05-10 switch runtime rendering to window swapchain present for RenderDoc capture
  - docs/superpowers/plans/2026-05-10-runtime-surface-present.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
tests:
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - cargo test -p zircon_runtime --locked --verbose dynamic_api
  - cargo test -p zircon_runtime --locked --verbose graphics_surface
  - cargo test -p zircon_runtime --locked --verbose render_debugger
  - cargo test -p zircon_app --locked --verbose runtime_entry
  - cargo test -p zircon_app --locked --verbose
  - cargo test -p zircon_runtime_interface --locked --verbose
  - cargo check -p zircon_runtime --locked
  - cargo check -p zircon_app --locked
doc_type: module-detail
---

# Window Swapchain Present

## Ownership

`zircon_app` owns process entry, winit windows, raw native handles, and fallback presentation. It does not depend on wgpu and does not construct renderer objects. The app extracts a `ZrRuntimeNativeSurfaceTargetV1` from the active winit window, asks the loaded runtime whether the optional surface-present ABI is available, and then prefers `bind_viewport_surface` plus `present_viewport` on redraw. If the runtime lacks those fields, rejects the target, or present fails, the app disables surface present and falls back to `capture_frame` plus `SoftbufferRuntimePresenter`. The app writes scoped diagnostics for the transition points: `runtime_surface_present_enabled`, `runtime_surface_present_fallback`, and `runtime_surface_present_failed`.

`zircon_runtime` owns the rendering surface lifecycle. The dynamic ABI validates viewport, ABI version, size, and native target data, maps the ABI DTO into `RenderViewportSurfaceDescriptor`, and delegates through `RuntimeRenderBridge` into the `RenderFramework` trait. The framework contract is backend-neutral: it exposes bind, unbind, and present-extract methods without leaking winit or raw-window-handle types to app code.

The neutral `RenderFramework` default implementation treats viewport surface presentation as an optional capability. Binding and presenting return `RenderFrameworkError::UnsupportedCapability { capability: "viewport surface present" }`, while default unbind is a no-op so callers can safely clean up best-effort even when a backend never supported surface presentation. The WGPU implementation validates unknown viewports before attempting native surface creation, and a present request without a bound viewport surface reports the same unsupported capability instead of disguising the condition as an opaque backend failure.

## Render Flow

The window path still renders the full scene through the existing extract, pipeline, runtime-prepare, history, feedback, overlay, and UI path. The difference is the final step. `present_frame_extract` renders into the renderer's offscreen final color target, then `ViewportSurface::present_texture` acquires a wgpu `SurfaceTexture`, draws a full-screen blit from the offscreen final color view into the swapchain texture, submits that GPU work, and calls `SurfaceTexture::present()`.

This first window-present slice deliberately keeps the offscreen render target as the product render target. That preserves existing post-process, history, UI composition, capture, and readback behavior while removing the CPU readback from the live window redraw path. It also gives RenderDoc a real present boundary, so captures should report the runtime as presenting instead of `D3D12 (Not Presenting)` when the window path is active.

## Surface Details

The initial native target implementation is Win32 because the current app-side raw handle extraction only emits Win32 handles. On Windows, the runtime creates a wgpu unsafe raw-handle surface from `HWND` plus optional `HINSTANCE`; on other native targets, the same descriptor returns a scoped surface-status error before raw Win32 handle construction. The backend queries the existing adapter's surface capabilities, configures the surface with a preferred advertised BGRA/RGBA format, chooses an advertised present mode with `AutoVsync` preferred and `Fifo` used only when present, rejects surfaces that advertise no present modes, clamps zero descriptor dimensions to at least `1x1`, and uses a frame latency of two. The present blit uses a small wgpu pipeline so the swapchain format does not need to match Zircon's `Rgba8UnormSrgb` offscreen format.

Surface loss, outdated swapchains, occlusion, and timeout are treated as non-fatal present skips. Validation errors and surface creation/configuration failures are returned to the dynamic API, causing the app to fall back to softbuffer readback presentation.

## Runtime Preview Launch

The runtime-preview Cargo binary is `zircon_runtime` under the `zircon_app` package. A manual Windows RenderDoc run should use the same environment before process start:

```powershell
$env:WGPU_BACKEND='dx12'
$env:WGPU_DEBUG='1'
$env:WGPU_VALIDATION='1'
cargo run -p zircon_app --bin zircon_runtime --locked
```

The ordinary RenderDoc `Capture Frame` path is expected to capture the runtime-preview `WindowSurface` path with a swapchain present boundary after `zircon-present-blit-pass`. The older `OffscreenReadback` path is still expected when native handle extraction, optional ABI availability, surface bind, surface creation/configuration, or present fails; it is also still the editor viewport path until a separate editor GPU embedding milestone.

## Validation Notes

The automated coverage verifies that the dynamic runtime exports the optional surface-present functions, rejects wrong ABI and unknown viewport requests before session lookup, validates target ABI before session lookup, converts ABI surface descriptors through `zircon_runtime/src/dynamic_api/surface.rs`, returns `runtime session not found` for an invalid session with an otherwise valid Win32 descriptor, and keeps `capture_frame()` wrong-ABI and unknown-viewport rejection independent of rendering. The app runtime loader only treats the window-present path as supported when all optional ABI fields are present and inside the reported table size. The app source-level `runtime_entry` coverage checks that failed surface-present state gates redraw, that enabled/fallback/failed diagnostic strings remain present, and that `present_viewport`, `capture_frame`, `SoftbufferRuntimePresenter::new`, and `about_to_wait` redraw driving stay wired. `cargo test -p zircon_runtime --locked --verbose graphics_surface` also verifies the framework default unsupported/no-op contract, WGPU unknown viewport validation, missing bound surface diagnostics, capture-pending cleanup when present is rejected before a surface is available, no captured-frame counter increment on missing-surface present, backend descriptor-size clamping, backend format and advertised present-mode selection including empty-list rejection, panic-safe restoration of leased viewport surface slots, a source-level guard that the present path calls `SurfaceTexture::present()` without `read_texture_rgba()` fallback, and that offscreen submit plus `capture_frame()` remains usable after surface unbind. Manual acceptance still requires launching the runtime window under RenderDoc and confirming that RenderDoc reports a presenting API and that the timeline ends in a swapchain present after the `zircon-present-blit-pass`.

The 2026-05-11 Milestone 2 validation ran `cargo test -p zircon_runtime --locked --verbose graphics_surface` and `cargo check -p zircon_runtime --locked`. The refreshed filtered test run passed with twelve selected tests, and the runtime check passed after waiting on build/package locks. The 2026-05-12 Milestone 3 validation ran `cargo test -p zircon_runtime --locked --verbose graphics_surface`, `cargo test -p zircon_runtime --locked --verbose render_debugger`, and `cargo check -p zircon_runtime --locked`; all passed after the backend helper rejected empty present-mode capability lists instead of defaulting to an unadvertised mode. The 2026-05-12 Milestone 4 validation ran `cargo test -p zircon_runtime --locked --verbose dynamic_api` and `cargo test -p zircon_runtime --locked --verbose graphics_surface`; both passed. The 2026-05-12 Milestone 5 app validation ran `cargo test -p zircon_app --locked --verbose runtime_entry`, which passed with `1 passed; 0 failed; 40 filtered out`, and `cargo check -p zircon_app --locked`, which passed. After spec review, the app source guard was tightened to cover initial resize-before-bind order, bind request construction, resize rebinding order, same-branch fallback after present failure, failure marking, and teardown unbind coverage; the same scoped app test/check commands passed again. Spec re-review and code-quality review then approved Milestone 5 with no findings.

The 2026-05-12 Milestone 6 validation ran `cargo test -p zircon_runtime_interface --locked --verbose`, which passed with `94 passed; 0 failed`; scoped runtime validation through `graphics_surface`, `dynamic_api`, and `render_debugger`, all of which passed; and `cargo test -p zircon_app --locked --verbose`, which passed after formatting with `41 passed; 0 failed` plus zero-test runtime-preview binary and doc-test runs. A later runtime closeout resolved the unrelated Material foundation `TreeView.expanded` catalog blocker, refreshed `zircon_plugins/Cargo.lock`, corrected neutral runtime-world default VG extraction, and preserved explicit empty authored VG extracts as clear-only frames. Focused VG/editor-boundary validation passed for `virtual_geometry_stats_contract`, `virtual_geometry_execution_snapshot_contract`, and `m1_runtime_editor_boundary_contract`, then broad `cargo test -p zircon_runtime --locked --verbose` passed with `1268 passed; 0 failed` in the runtime lib target plus green integration/doc-test targets. The full workspace validator passed the build phase and then failed in `zircon_editor --lib` on retained host template projection: `NameField.text` projected as `Some("Name")` while `host_projection_carries_runtime_component_properties_and_routes` expects `None`. That remaining blocker belongs to the active retained UI/template lane, not to runtime surface presentation. Manual Windows runtime-preview launch and RenderDoc ordinary capture acceptance are still pending because they require an interactive GPU/window capture session.
