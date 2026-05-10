# Runtime Surface Present Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real `wgpu::Surface` / swapchain present path for runtime preview so RenderDoc's ordinary frame capture can use a natural `SurfaceTexture::present()` boundary while preserving the current offscreen readback fallback.

**Architecture:** `zircon_app` remains the process and window host, `zircon_runtime_interface` owns the stable ABI descriptors for native surface binding, and `zircon_runtime::graphics` owns the concrete `wgpu::Surface` target and present logic. The first implementation targets the runtime preview window; editor viewport GPU embedding remains out of scope for this plan but keeps using the same framework direction.

**Tech Stack:** Rust workspace on `main`, `wgpu 29.0.1`, `winit 0.31.0-beta.2`, `raw-window-handle` through winit/window-handle APIs, `zircon_runtime_interface` stable ABI, existing `RenderFramework` and `WgpuRenderFramework`.

---

## Current Baseline

- `zircon_runtime::graphics` renders to `OffscreenTarget`, then `read_texture_rgba()` copies the final texture into CPU memory.
- Runtime preview in `zircon_app/src/entry/runtime_entry_app/application_handler.rs` calls `RuntimeSession::capture_frame()` and presents CPU pixels through `SoftbufferRuntimePresenter`.
- Editor retained host in `zircon_editor/src/ui/retained_host/host_contract/presenter.rs` uses `softbuffer` for the whole editor window, and editor viewport images are imported from captured CPU frames.
- RenderDoc in-app capture already exists via `Device::start_graphics_debugger_capture()` / `stop_graphics_debugger_capture()`, but ordinary RenderDoc capture lacks a swapchain present boundary.
- Runtime interface audit classifies `zircon_app` as converged and flags `zircon_runtime` / `zircon_editor` for large-file hotspots; this plan avoids expanding the existing large editor files.

## Target Boundaries

- `zircon_runtime_interface/src/runtime_api.rs` defines C-safe surface binding and present ABI types.
- `zircon_app/src/entry/runtime_entry_app/` extracts native window handles and chooses surface present or softbuffer fallback.
- `zircon_app/src/entry/runtime_library/runtime_session.rs` wraps optional runtime API calls and keeps existing `capture_frame()` fallback intact.
- `zircon_runtime/src/dynamic_api/` translates ABI surface requests into runtime graphics calls.
- `zircon_runtime/src/core/framework/render/` exposes a neutral render-framework surface target contract without leaking `winit` concrete types.
- `zircon_runtime/src/graphics/runtime/render_framework/` binds/unbinds viewport surface targets and routes surface presentation through existing pipeline preparation.
- `zircon_runtime/src/graphics/backend/render_backend/` owns unsafe `wgpu::Surface` creation, configuration, resize, acquire, present, and recovery.
- `docs/assets-and-rendering/render-framework-architecture.md` and a new mirrored docs page describe surface-present behavior and fallback.

## Milestone 1: Stable Runtime Surface ABI And App Fallback

**Goal:** Add optional ABI calls and app-side window handle extraction without changing rendering behavior yet.

**In-scope behaviors:** Windows native surface descriptor construction, optional function lookup, graceful fallback to `capture_frame()` when surface APIs are absent or unsupported, resize event propagation through both existing event path and new surface binding metadata.

**Dependencies:** Existing runtime dynamic API, `winit` window creation, `RuntimeSession`, existing softbuffer presenter.

**Implementation Slices:**

- [ ] Modify `zircon_runtime_interface/src/runtime_api.rs` to add C-safe surface ABI types:

```rust
pub const ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1: u32 = 0;
pub const ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1: u32 = 1;

pub type ZrRuntimeBindViewportSurfaceFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus;

pub type ZrRuntimeUnbindViewportSurfaceFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle,
) -> ZrStatus;

pub type ZrRuntimePresentViewportFnV1 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeFrameRequestV1,
) -> ZrStatus;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeNativeSurfaceTargetV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub window_handle: u64,
    pub display_handle: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeBindViewportSurfaceRequestV1 {
    pub abi_version: u32,
    pub viewport: ZrRuntimeViewportHandle,
    pub size: ZrRuntimeViewportSizeV1,
    pub target: ZrRuntimeNativeSurfaceTargetV1,
}
```

- [ ] Extend `ZrRuntimeApiV1` in `zircon_runtime_interface/src/runtime_api.rs` with optional fields after existing fields to preserve optional discovery:

```rust
pub bind_viewport_surface: Option<ZrRuntimeBindViewportSurfaceFnV1>,
pub unbind_viewport_surface: Option<ZrRuntimeUnbindViewportSurfaceFnV1>,
pub present_viewport: Option<ZrRuntimePresentViewportFnV1>,
```

- [ ] Add constructors in `zircon_runtime_interface/src/runtime_api.rs`:

```rust
impl ZrRuntimeNativeSurfaceTargetV1 {
    pub const fn none(abi_version: u32) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1,
            window_handle: 0,
            display_handle: 0,
        }
    }

    pub const fn win32(abi_version: u32, hwnd: u64, hinstance: u64) -> Self {
        Self {
            abi_version,
            kind: ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1,
            window_handle: hwnd,
            display_handle: hinstance,
        }
    }
}

impl ZrRuntimeBindViewportSurfaceRequestV1 {
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        target: ZrRuntimeNativeSurfaceTargetV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            target,
        }
    }
}
```

- [ ] Update `zircon_runtime_interface/src/lib.rs` re-exports for new constants, structs, and function types.
- [ ] Update `zircon_runtime_interface/src/tests/contracts.rs` to assert ABI default fields are `None`, constructors preserve handles, and `size_of::<ZrRuntimeApiV1>()` contract expectations account for appended function pointers.
- [ ] Add `zircon_app/src/entry/runtime_entry_app/window_surface.rs` with one responsibility: convert a `winit::window::Window` into `Option<ZrRuntimeNativeSurfaceTargetV1>`.

```rust
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::Window;
use zircon_runtime_interface::{
    ZrRuntimeNativeSurfaceTargetV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

pub(super) fn runtime_native_surface_target(
    window: &dyn Window,
) -> Option<ZrRuntimeNativeSurfaceTargetV1> {
    let window_handle = window.window_handle().ok()?.as_raw();
    let display_handle = window.display_handle().ok()?.as_raw();
    match (window_handle, display_handle) {
        (RawWindowHandle::Win32(window), RawDisplayHandle::Windows(display)) => Some(
            ZrRuntimeNativeSurfaceTargetV1::win32(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                window.hwnd.get() as u64,
                display.hinstance.map(|value| value.get() as u64).unwrap_or(0),
            ),
        ),
        _ => None,
    }
}
```

- [ ] Modify `zircon_app/src/entry/runtime_entry_app/mod.rs` to declare `window_surface` and keep `mod.rs` structural.
- [ ] Modify `zircon_app/src/entry/runtime_library/runtime_session.rs` with optional wrappers:

```rust
pub(crate) fn bind_viewport_surface(
    &self,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> Result<bool, RuntimeLibraryError> {
    let Some(bind) = self.runtime.api().bind_viewport_surface else {
        return Ok(false);
    };
    ensure_status(unsafe { bind(self.handle, request) }, "bind runtime viewport surface")?;
    Ok(true)
}

pub(crate) fn present_viewport(
    &self,
    viewport: ZrRuntimeViewportHandle,
    size: ZrRuntimeViewportSizeV1,
) -> Result<bool, RuntimeLibraryError> {
    let Some(present) = self.runtime.api().present_viewport else {
        return Ok(false);
    };
    ensure_status(
        unsafe {
            present(
                self.handle,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
            )
        },
        "present runtime viewport",
    )?;
    Ok(true)
}
```

- [ ] Modify `zircon_app/src/entry/runtime_entry_app/RuntimeEntryApp` state to store `surface_present_enabled: bool` and keep `presenter: Option<SoftbufferRuntimePresenter>` as fallback.
- [ ] Modify `zircon_app/src/entry/runtime_entry_app/application_handler.rs` so window creation tries `bind_viewport_surface()` first, creates `SoftbufferRuntimePresenter` only when binding returns `false`, and redraw calls `present_viewport()` when `surface_present_enabled` is true.
- [ ] Unit-test app fallback behavior by adding focused tests under `zircon_app/src/entry/tests/` using a fake `RuntimeSession`-like seam only if an existing seam exists; otherwise add tests at `runtime_session.rs` level for optional function absence.

**Testing Stage:**

- [ ] Run `cargo test -p zircon_runtime_interface --locked --verbose`.
- [ ] Run `cargo test -p zircon_app --locked --verbose`.
- [ ] Debug failures in ABI layout, optional call wrappers, or feature gates before promoting.
- [ ] Acceptance evidence: app still compiles with runtime API functions absent, and existing softbuffer path remains callable.

**Lightweight Checks:**

- [ ] During implementation, use `cargo check -p zircon_runtime_interface --locked` and `cargo check -p zircon_app --locked` only after the milestone slice compiles locally enough to need type evidence.

## Milestone 2: Render Framework Surface Target Contract

**Goal:** Add render-framework and viewport-record concepts for bound surface targets without presenting yet.

**In-scope behaviors:** Surface target request validation, unknown viewport errors, bind/unbind state transitions, offscreen fallback unchanged, no concrete `winit` types in framework contracts.

**Dependencies:** Milestone 1 ABI types.

**Implementation Slices:**

- [ ] Modify `zircon_runtime/src/core/framework/render/backend_types.rs` with neutral contract types:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderNativeSurfaceTarget {
    Win32 { hwnd: u64, hinstance: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderViewportSurfaceDescriptor {
    pub size: UVec2,
    pub target: RenderNativeSurfaceTarget,
}
```

- [ ] Modify `zircon_runtime/src/core/framework/render/framework.rs` to add default no-op methods returning `Unsupported` or `Ok(false)` according to existing `RenderFrameworkError` variants. If `RenderFrameworkError` lacks an unsupported-capability variant, add `UnsupportedCapability { capability: String }` in `framework_error.rs`.

```rust
fn bind_viewport_surface(
    &self,
    _viewport: RenderViewportHandle,
    _descriptor: RenderViewportSurfaceDescriptor,
) -> Result<(), RenderFrameworkError> {
    Err(RenderFrameworkError::UnsupportedCapability {
        capability: "viewport surface present".to_string(),
    })
}

fn unbind_viewport_surface(
    &self,
    _viewport: RenderViewportHandle,
) -> Result<(), RenderFrameworkError> {
    Ok(())
}

fn present_viewport(
    &self,
    _viewport: RenderViewportHandle,
    _extract: RenderFrameExtract,
) -> Result<(), RenderFrameworkError> {
    Err(RenderFrameworkError::UnsupportedCapability {
        capability: "viewport surface present".to_string(),
    })
}
```

- [ ] Modify `zircon_runtime/src/core/framework/render/mod.rs` to re-export the new contract types.
- [ ] Modify `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs` to add:

```rust
pub(super) surface: Option<RenderViewportSurfaceDescriptor>,
```

- [ ] Modify `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/new.rs` so `surface` starts as `None`.
- [ ] Add `zircon_runtime/src/graphics/runtime/render_framework/surface_target/mod.rs`, `bind.rs`, `unbind.rs`, and `present.rs` as focused modules.
- [ ] Implement `bind_viewport_surface()` to validate viewport existence and store descriptor on the `ViewportRecord`.
- [ ] Implement `unbind_viewport_surface()` to clear descriptor and ask the backend to release concrete surface resources in Milestone 3.
- [ ] Implement a temporary `present_viewport()` framework path that validates the viewport has a surface descriptor and returns `UnsupportedCapability` until backend present is wired in Milestone 3.
- [ ] Modify `zircon_runtime/src/graphics/runtime/render_framework/render_framework_impl/trait_impl.rs` to delegate new trait methods into `surface_target` modules.
- [ ] Add unit tests in `zircon_runtime/src/graphics/tests/render_debugger_and_history.rs` or a new focused `zircon_runtime/src/graphics/tests/surface_targets.rs` for unknown viewport, bind state, unbind state, and offscreen `capture_frame()` unaffected after bind/unbind.

**Testing Stage:**

- [ ] Run `cargo test -p zircon_runtime --locked --verbose graphics_surface` after naming focused tests with `graphics_surface` prefix.
- [ ] Run `cargo check -p zircon_runtime --locked` if the focused test filter is too narrow for trait object coverage.
- [ ] Debug lower contract errors before touching backend present.
- [ ] Acceptance evidence: API stores surface descriptors safely and existing offscreen submit/capture tests still pass.

**Lightweight Checks:**

- [ ] Use `cargo check -p zircon_runtime --locked` once after adding trait methods and before backend wiring if compiler errors become hard to reason about from source review.

## Milestone 3: WGPU Surface Backend And Present Path

**Goal:** Implement concrete `wgpu::Surface` creation, configuration, resize, frame acquire, render, submit, and present.

**In-scope behaviors:** Windows native surface creation, surface format selection, resize reconfiguration, lost/outdated surface recovery, timeout/out-of-memory error reporting, full frame render into swapchain texture, no CPU readback on surface present path.

**Dependencies:** Milestone 2 surface target state and framework methods.

**Implementation Slices:**

- [ ] Add `zircon_runtime/src/graphics/backend/render_backend/surface_target/mod.rs` with structural exports only.
- [ ] Add `surface_key.rs` containing a `RenderSurfaceKey` derived from viewport handle raw value so backend resources can be looked up without depending on framework record internals.
- [ ] Add `surface_target.rs` containing:

```rust
pub(crate) struct BackendSurfaceTarget {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: UVec2,
}
```

- [ ] Add `create.rs` to convert `RenderNativeSurfaceTarget::Win32` into `wgpu::SurfaceTargetUnsafe::RawHandle` using `raw_window_handle` raw handle values and `unsafe { instance.create_surface_unsafe(...) }`.
- [ ] Add `configure.rs` to choose `TextureFormat` from `surface.get_capabilities(&adapter)`, prefer an sRGB format when present, configure usage with `RENDER_ATTACHMENT`, and use FIFO present mode when available.
- [ ] Add `acquire.rs` to handle `SurfaceError::Lost` and `SurfaceError::Outdated` by reconfiguring once, return `GraphicsError` for `OutOfMemory`, and skip empty zero-size inputs by clamping to at least `1x1`.
- [ ] Modify `zircon_runtime/src/graphics/backend/render_backend/render_backend.rs` to add:

```rust
pub(crate) surface_targets: std::collections::HashMap<RenderSurfaceKey, BackendSurfaceTarget>,
```

- [ ] Modify `render_backend_new_offscreen.rs` to initialize `surface_targets: HashMap::new()`.
- [ ] Add `RenderBackend::bind_surface_target()`, `unbind_surface_target()`, and `surface_frame_view()` helpers in the new `surface_target` module; keep `render_backend/mod.rs` structural.
- [ ] Modify `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs` by extracting the shared pass recording into a target-view oriented helper only if needed; do not duplicate the full render graph path.
- [ ] Add a new focused path `SceneRenderer::present_frame_with_pipeline()` in `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/present_frame_with_pipeline.rs` that mirrors `render_frame_with_pipeline()` but targets the acquired surface texture and calls `surface_texture.present()` after `queue.submit()`.
- [ ] Ensure `present_frame_with_pipeline()` updates runtime outputs, generation, stats, history, and debugger finish state consistently with `render_frame_with_pipeline()`.
- [ ] Modify `zircon_runtime/src/graphics/runtime/render_framework/surface_target/present.rs` to build the same `FrameSubmissionContext`, prepare runtime sidebands, resolve history, call `present_frame_with_pipeline()`, record submission metadata without `last_capture`, and update stats.
- [ ] Add unit tests for backend-independent behavior: bind/present rejects unknown viewport, present rejects missing surface, resize updates descriptor, and surface present does not increment captured-frame counters.
- [ ] Add `#[cfg(target_os = "windows")]` around Win32 raw handle conversion and return `UnsupportedCapability` on other platforms.

**Testing Stage:**

- [ ] Run `cargo test -p zircon_runtime --locked --verbose graphics_surface`.
- [ ] Run `cargo test -p zircon_runtime --locked --verbose render_debugger` to ensure in-app RenderDoc capture state still works.
- [ ] Run `cargo check -p zircon_runtime --locked` for target-gated surface code.
- [ ] Debug rendering backend failures from lowest layer first: ABI target conversion, surface creation, surface configuration, acquire, render pass recording, present.
- [ ] Acceptance evidence: surface path reaches `SurfaceTexture::present()` on Windows runtime preview, and offscreen readback tests remain green.

**Lightweight Checks:**

- [ ] During implementation, run only `cargo check -p zircon_runtime --locked` when unsafe surface code or lifetime changes produce type uncertainty.

## Milestone 4: Runtime Dynamic API Integration

**Goal:** Connect runtime ABI calls to render framework surface binding and presentation.

**In-scope behaviors:** ABI validation, session lookup, viewport conversion, surface target conversion, present frame request path, clear diagnostics for unsupported surface kinds, existing `capture_frame()` preserved.

**Dependencies:** Milestones 1 through 3.

**Implementation Slices:**

- [ ] Add `zircon_runtime/src/dynamic_api/surface.rs` with conversion functions:

```rust
fn render_surface_descriptor(
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> Result<RenderViewportSurfaceDescriptor, String> {
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err("unsupported runtime surface request ABI version".to_string());
    }
    let target = match request.target.kind {
        ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1 => RenderNativeSurfaceTarget::Win32 {
            hwnd: request.target.window_handle,
            hinstance: request.target.display_handle,
        },
        _ => return Err("unsupported runtime native surface target".to_string()),
    };
    Ok(RenderViewportSurfaceDescriptor {
        size: UVec2::new(request.size.width.max(1), request.size.height.max(1)),
        target,
    })
}
```

- [ ] Modify `zircon_runtime/src/dynamic_api/session.rs` to add unsafe extern functions `bind_viewport_surface`, `unbind_viewport_surface`, and `present_viewport` that call `with_session()`.
- [ ] Add methods to `RuntimeDynamicSession`:

```rust
fn bind_viewport_surface(
    &mut self,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus;

fn unbind_viewport_surface(&mut self, viewport: ZrRuntimeViewportHandle) -> ZrStatus;

fn present_viewport(&mut self, request: ZrRuntimeFrameRequestV1) -> ZrStatus;
```

- [ ] In `present_viewport`, build the same scene extract currently used by `capture_frame()` and call the new render framework `present_viewport()` instead of `submit_frame_extract()` + `capture_frame()`.
- [ ] Modify `zircon_runtime/src/dynamic_api/exports.rs` to populate the new optional function pointers.
- [ ] Modify `zircon_runtime/src/dynamic_api/tests.rs` to assert the API exposes the new optional functions and unsupported surface target returns `ZrStatusCode::InvalidArgument` with diagnostics.
- [ ] Add a dynamic API test where `bind_viewport_surface` on an invalid session returns `NotFound` and does not panic.

**Testing Stage:**

- [ ] Run `cargo test -p zircon_runtime --locked --verbose dynamic_api`.
- [ ] Run `cargo test -p zircon_runtime --locked --verbose graphics_surface`.
- [ ] Debug correction loop covers status conversion, session lookup, render framework method dispatch, and fallback capture compatibility.
- [ ] Acceptance evidence: dynamic API reports surface functions as available and preserves existing `capture_frame()` behavior.

**Lightweight Checks:**

- [ ] Use `cargo check -p zircon_runtime --locked` after function pointer and ABI export changes.

## Milestone 5: App Runtime Preview Surface Present Switch

**Goal:** Use the new runtime API from the real runtime preview window and make ordinary RenderDoc capture possible.

**In-scope behaviors:** Bind surface after window creation, rebind or resize on `SurfaceResized`, present on redraw without softbuffer when supported, fallback to softbuffer readback when binding or present returns unsupported, unbind on session/window teardown when available.

**Dependencies:** Milestones 1 through 4.

**Implementation Slices:**

- [ ] Modify `zircon_app/src/entry/runtime_entry_app/runtime_entry_app.rs` or the existing struct declaration file to store:

```rust
pub(super) surface_present_enabled: bool,
pub(super) surface_present_failed: bool,
```

- [ ] In `can_create_surfaces`, after `resize_viewport`, call `runtime_native_surface_target(window.as_ref())` and `RuntimeSession::bind_viewport_surface()` with `ZrRuntimeBindViewportSurfaceRequestV1::new(...)`.
- [ ] Create `SoftbufferRuntimePresenter` only when native surface target extraction fails, bind returns `false`, or bind returns an unsupported/capability-denied status.
- [ ] In `WindowEvent::SurfaceResized`, call both `resize_viewport()` and `bind_viewport_surface()` again when `surface_present_enabled` is true so the runtime backend reconfigures the swapchain size.
- [ ] In `WindowEvent::RedrawRequested`, branch:

```rust
if self.surface_present_enabled && !self.surface_present_failed {
    match self.session.present_viewport(self.viewport, self.viewport_size) {
        Ok(true) => return,
        Ok(false) => self.surface_present_failed = true,
        Err(_) => self.surface_present_failed = true,
    }
}
// Existing capture_frame + softbuffer fallback remains here.
```

- [ ] If surface present fails at runtime, initialize `SoftbufferRuntimePresenter` on demand and fall back to existing `capture_frame()` present in the same redraw branch.
- [ ] Add diagnostic log entries for `runtime_surface_present_enabled`, `runtime_surface_present_fallback`, and `runtime_surface_present_failed` using existing diagnostic log utilities if available in `zircon_app`; otherwise keep errors scoped to existing runtime app error handling.
- [ ] Ensure `about_to_wait()` continues requesting redraw so surface present is driven frame-by-frame.

**Testing Stage:**

- [ ] Run `cargo test -p zircon_app --locked --verbose runtime_entry`.
- [ ] Run `cargo check -p zircon_app --locked`.
- [ ] Run a manual Windows runtime preview smoke command from repository root:

```powershell
$env:WGPU_BACKEND='dx12'; $env:WGPU_DEBUG='1'; $env:WGPU_VALIDATION='1'; cargo run -p zircon_app --bin runtime_preview --locked
```

- [ ] In RenderDoc, launch the runtime preview process with the same environment and verify the ordinary `Capture Frame` button captures a frame with a swapchain present boundary.
- [ ] Debug correction loop: if app falls back to softbuffer, inspect diagnostic status in this order: native handle extraction, ABI function availability, runtime surface bind, backend surface creation, present call.

**Lightweight Checks:**

- [ ] Use `cargo check -p zircon_app --locked` after app state and event-loop changes.

## Milestone 6: Documentation And Workspace Acceptance

**Goal:** Document the surface-present architecture, capture modes, fallback behavior, and validation evidence.

**In-scope behaviors:** Module docs with related-code headers, RenderDoc usage notes, validation records, offscreen fallback explanation, editor viewport scope statement.

**Dependencies:** Milestones 1 through 5.

**Implementation Slices:**

- [ ] Update `docs/assets-and-rendering/render-framework-architecture.md` frontmatter to include new related files and describe `OffscreenReadback` versus `WindowSurface` viewport targets.
- [ ] Add `docs/zircon_runtime/graphics/surface-present.md` with frontmatter:

```markdown
---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/backend/render_backend/render_backend.rs
  - zircon_runtime/src/graphics/backend/render_backend/surface_target/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/surface_target/mod.rs
  - zircon_runtime/src/dynamic_api/surface.rs
implementation_files:
  - zircon_runtime/src/graphics/backend/render_backend/surface_target/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/surface_target/mod.rs
  - zircon_runtime/src/dynamic_api/surface.rs
plan_sources:
  - user: 2026-05-10 Add true wgpu Surface present path for RenderDoc ordinary capture.
tests:
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---
```

- [ ] Add `docs/zircon_app/runtime-surface-present.md` with frontmatter covering app window binding and softbuffer fallback.
- [ ] Document RenderDoc launch recipe:

```powershell
$env:WGPU_BACKEND='dx12'
$env:WGPU_DEBUG='1'
$env:WGPU_VALIDATION='1'
cargo run -p zircon_app --bin runtime_preview --locked
```

- [ ] Document acceptance distinction: ordinary RenderDoc capture is expected for runtime preview surface path; editor viewport remains on offscreen readback until the editor GPU embedding milestone.

**Testing Stage:**

- [ ] Run `cargo test -p zircon_runtime_interface --locked --verbose`.
- [ ] Run `cargo test -p zircon_runtime --locked --verbose`.
- [ ] Run `cargo test -p zircon_app --locked --verbose`.
- [ ] Run full workspace validation before final closeout:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

- [ ] Record validation evidence in final response and docs if failures required code changes.
- [ ] Acceptance evidence: runtime preview presents through `SurfaceTexture::present()`, softbuffer fallback still works when surface APIs are unavailable, tests pass, and docs map all affected code.

**Lightweight Checks:**

- [ ] No additional lightweight check is needed after docs-only changes; defer full verification to this milestone testing stage.

## Risks And Guardrails

- Unsafe surface creation must stay in `zircon_runtime/src/graphics/backend/render_backend/surface_target/`; no unsafe raw window handle code belongs in editor UI or app event-loop files beyond handle extraction.
- `zircon_runtime_interface` must remain C-safe. Do not expose Rust enums, `raw_window_handle` types, references, `Arc`, or trait objects across the dynamic API.
- Surface present must not remove `capture_frame()`. Tests, headless runs, unsupported platforms, and editor viewport still need readback fallback.
- If wgpu surface lifetimes require owning handles differently than the ABI descriptor allows, stop and redesign the surface ownership boundary rather than storing borrowed window references in runtime.
- Do not expand `zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs` or other large editor hotspots for this milestone.

## Plan Self-Review

- Spec coverage: runtime ordinary RenderDoc capture is covered by Milestones 3 and 5; fallback and in-app capture preservation are covered by Milestones 1, 2, and 6; docs are covered by Milestone 6.
- Placeholder scan: the plan contains concrete files, method names, commands, tests, and acceptance evidence; it avoids unspecified implementation slots.
- Type consistency: ABI types use `ZrRuntime*V1`; framework types use `Render*`; backend types remain crate-private under `graphics/backend`.
