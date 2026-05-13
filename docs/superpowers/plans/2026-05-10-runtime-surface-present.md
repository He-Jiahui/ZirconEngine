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

- [x] Modify `zircon_runtime_interface/src/runtime_api.rs` to add C-safe surface ABI types:

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

- [x] Extend `ZrRuntimeApiV1` in `zircon_runtime_interface/src/runtime_api.rs` with optional fields after existing fields to preserve optional discovery:

```rust
pub bind_viewport_surface: Option<ZrRuntimeBindViewportSurfaceFnV1>,
pub unbind_viewport_surface: Option<ZrRuntimeUnbindViewportSurfaceFnV1>,
pub present_viewport: Option<ZrRuntimePresentViewportFnV1>,
```

- [x] Add constructors in `zircon_runtime_interface/src/runtime_api.rs`:

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

- [x] Update `zircon_runtime_interface/src/lib.rs` re-exports for new constants, structs, and function types.
- [x] Update `zircon_runtime_interface/src/tests/contracts.rs` to assert ABI default fields are `None`, constructors preserve handles, and `size_of::<ZrRuntimeApiV1>()` contract expectations account for appended function pointers.
- [x] Add `zircon_app/src/entry/runtime_entry_app/window_surface.rs` with one responsibility: convert a `winit::window::Window` into `Option<ZrRuntimeNativeSurfaceTargetV1>`.

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
        (RawWindowHandle::Win32(window), RawDisplayHandle::Windows(_display)) => Some(
            ZrRuntimeNativeSurfaceTargetV1::win32(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                window.hwnd.get() as usize as u64,
                window.hinstance.map(|value| value.get() as usize as u64).unwrap_or(0),
            ),
        ),
        _ => None,
    }
}
```

- [x] Modify `zircon_app/src/entry/runtime_entry_app/mod.rs` to declare `window_surface` and keep `mod.rs` structural.
- [x] Modify `zircon_app/src/entry/runtime_library/runtime_session.rs` with optional wrappers:

```rust
pub(crate) fn bind_viewport_surface(
    &self,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> Result<bool, RuntimeLibraryError> {
    let Some(bind) = self.runtime.bind_viewport_surface() else {
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
    let Some(present) = self.runtime.present_viewport() else {
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

pub(crate) fn supports_viewport_surface_present(&self) -> bool {
    self.runtime.supports_viewport_surface_present()
}
```

- [x] Modify `zircon_app/src/entry/runtime_entry_app/RuntimeEntryApp` state to store `surface_present_enabled: bool` and keep `presenter: Option<SoftbufferRuntimePresenter>` as fallback.
- [x] Modify `zircon_app/src/entry/runtime_entry_app/application_handler.rs` so window creation tries `bind_viewport_surface()` first, creates `SoftbufferRuntimePresenter` only when binding returns `false`, and redraw calls `present_viewport()` when `surface_present_enabled` is true.
- [x] Unit-test app fallback behavior by adding focused tests under `zircon_app/src/entry/tests/` using a fake `RuntimeSession`-like seam only if an existing seam exists; otherwise add tests at `runtime_session.rs` level for optional function absence.

**Testing Stage:**

- [x] Run `cargo test -p zircon_runtime_interface --locked --verbose`.
- [x] Run `cargo test -p zircon_app --locked --verbose`.
- [x] Confirm no ABI layout, optional call wrapper, or feature-gate failures remained before promoting.
- [x] Acceptance evidence: app still compiles with runtime API functions absent, and existing softbuffer path remains callable.

**2026-05-11 Initial Milestone 1 Evidence:** `cargo fmt --all --check` passed with no output. `cargo test -p zircon_runtime_interface --locked --verbose` passed with `91 passed; 0 failed`. `cargo test -p zircon_app --locked --verbose` passed with `39 passed; 0 failed`. App-side fallback gated appended ABI fields by `ZrRuntimeApiV1::size_bytes`, required bind/unbind/present to be coherently present before enabling surface presentation, bound/rebound native metadata before resize events in the surface path, unbound before softbuffer fallback after surface-present failure, and handled softbuffer presenter creation failures without panicking.

**2026-05-11 Blocker Follow-up:** `LoadedRuntime` now stores the raw API pointer plus validated `size_bytes`, validates the required prefix before reading required function fields, and exposes only offset-gated function accessors instead of a full-width runtime API reference. `RuntimeEntryApp` tracks attempted surface binds and best-effort unbinds during `Drop` before window/presenter teardown. `cargo test -p zircon_app --locked --verbose` reached `40 passed; 0 failed` after Cargo generated unrelated `taffy` lock entries from the active UI lane; those generated lock entries were removed to preserve unrelated dirty work, and the final clean-lock rerun is blocked before compile by the unrelated `zircon_runtime/Cargo.toml` / `Cargo.lock` mismatch. `cargo fmt --all --check` is currently blocked by unrelated active UI formatting drift in `zircon_runtime/src/ui/tests/v2_asset.rs` and `zircon_runtime/src/ui/v2/surface_builder.rs`.

**Lightweight Checks:**

- [ ] During implementation, use `cargo check -p zircon_runtime_interface --locked` and `cargo check -p zircon_app --locked` only after the milestone slice compiles locally enough to need type evidence.

## Milestone 2: Render Framework Surface Target Contract

**Goal:** Confirm and tighten the render-framework and viewport-record contracts for bound surface targets without regressing the more advanced present path already present in current `main`.

**In-scope behaviors:** Surface target request validation, unknown viewport errors, bind/unbind state transitions, missing-surface diagnostics, offscreen fallback unchanged, no concrete `winit` types in framework contracts, and preserving any already-landed `present_frame_extract` implementation instead of downgrading it to a placeholder.

**Dependencies:** Milestone 1 ABI types.

**Implementation Slices:**

- [x] Add neutral contract types under `zircon_runtime/src/core/framework/render/` (implemented in the current tree as `surface.rs`):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderNativeSurfaceTarget {
    Win32 { hwnd: u64, hinstance: Option<u64> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderViewportSurfaceDescriptor {
    pub size: UVec2,
    pub target: RenderNativeSurfaceTarget,
}
```

- [x] Modify `zircon_runtime/src/core/framework/render/framework.rs` to add default surface methods returning `UnsupportedCapability` for bind/present and `Ok(())` for best-effort unbind. If `RenderFrameworkError` lacks an unsupported-capability variant, add `UnsupportedCapability { capability: String }` in `framework_error.rs`.

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

fn present_frame_extract(
    &self,
    _viewport: RenderViewportHandle,
    _extract: RenderFrameExtract,
) -> Result<(), RenderFrameworkError> {
    Err(RenderFrameworkError::UnsupportedCapability {
        capability: "viewport surface present".to_string(),
    })
}
```

- [x] Modify `zircon_runtime/src/core/framework/render/mod.rs` to re-export the new contract types.
- [x] Modify `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/viewport_record.rs` to add viewport-bound surface state:

```rust
pub(super) surface: Option<ViewportSurface>,
```

- [x] Modify `zircon_runtime/src/graphics/runtime/render_framework/viewport_record/new.rs` so `surface` starts as `None`.
- [x] Add focused runtime render-framework surface modules (implemented in the current tree as `viewport_surface/mod.rs` and `viewport_surface/viewport_surface.rs`).
- [x] Implement `bind_viewport_surface()` to validate viewport existence and store surface state on the `ViewportRecord`.
- [x] Implement `unbind_viewport_surface()` to clear viewport surface state.
- [x] Implement `present_frame_extract()` validation so a missing bound surface returns `UnsupportedCapability`; preserve the already-landed WGPU present path instead of replacing it with a Milestone 2 placeholder.
- [x] Modify `zircon_runtime/src/graphics/runtime/render_framework/render_framework_impl/trait_impl.rs` to delegate bind/unbind into `viewport_surface` modules and present into the existing `submit_frame_extract` present path.
- [x] Add unit tests in `zircon_runtime/src/graphics/tests/surface_targets.rs` for default unsupported behavior, unknown viewport handling, missing-surface present diagnostics, and offscreen `capture_frame()` unaffected after unbind.

**Testing Stage:**

- [x] Run `cargo test -p zircon_runtime --locked --verbose graphics_surface` after naming focused tests with `graphics_surface` prefix.
- [x] Run `cargo check -p zircon_runtime --locked` if the focused test filter is too narrow for trait object coverage.
- [x] Debug lower contract errors before touching backend present.
- [x] Acceptance evidence: API stores viewport surface state safely, reports missing bound surfaces explicitly, and existing offscreen submit/capture tests still pass.

**2026-05-11 Milestone 2 Evidence:** The current tree already carried neutral render surface DTOs in `zircon_runtime/src/core/framework/render/surface.rs`, WGPU viewport surface modules under `zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/`, and a present path under `submit_frame_extract`. This pass aligned the framework contract with an explicit `RenderFrameworkError::UnsupportedCapability`, made default unbind a no-op, made missing bound surface present report `viewport surface present` as unsupported, and added `zircon_runtime/src/graphics/tests/surface_targets.rs`. `cargo test -p zircon_runtime --locked --verbose graphics_surface` passed with `9 passed; 0 failed; 1232 filtered out`. `cargo check -p zircon_runtime --locked` passed after waiting on build/package locks.

**Lightweight Checks:**

- [ ] Use `cargo check -p zircon_runtime --locked` once after adding trait methods and before backend wiring if compiler errors become hard to reason about from source review.

## Milestone 3: WGPU Surface Backend And Present Path

**Goal:** Verify and complete the concrete `wgpu::Surface` creation, configuration, frame acquire, render, submit, and present path that already exists in the current architecture.

**In-scope behaviors:** Windows native surface creation from ABI raw handles, target-gated unsupported-platform behavior, surface format and present-mode selection from advertised capabilities, descriptor-size clamping, lost/outdated surface recovery, timeout/occluded present skips, validation/error reporting, full frame render into the swapchain texture through `SurfaceTexture::present()`, and no CPU readback on the surface present path.

**Dependencies:** Milestone 2 surface target state and framework methods.

**Implementation Slices:**

- [x] Preserve the current architecture instead of introducing the older planned `surface_target` directory: backend concrete surface code lives in `zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs`, render-framework bind/unbind code lives under `zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/`, and present is routed through `submit_frame_extract/submit/present_frame_extract.rs`.
- [x] Verify `RenderBackend::create_viewport_surface()` creates `wgpu::Surface<'static>` from `RenderNativeSurfaceTarget::Win32` using `wgpu::SurfaceTargetUnsafe::RawHandle` and `instance.create_surface_unsafe(...)` without leaking winit/app types into runtime graphics.
- [x] Add a target gate around Win32 raw handle construction so non-Windows builds return a scoped surface-status error instead of compiling or executing Win32 raw-handle construction unconditionally.
- [x] Verify `configure_surface()` queries `surface.get_capabilities(&adapter)`, selects an advertised BGRA/RGBA sRGB format when available, clamps descriptor size to at least `1x1`, configures `RENDER_ATTACHMENT`, and stores the resulting `wgpu::SurfaceConfiguration` on `ViewportSurface`.
- [x] Tighten present-mode selection so `AutoVsync` remains preferred when advertised, `Fifo` is used only when advertised, and otherwise the backend falls back to the first advertised mode instead of inventing an unsupported present mode.
- [x] Verify `ViewportSurface::present_texture()` handles `Success` and `Suboptimal` by blitting to the acquired surface texture and calling `SurfaceTexture::present()`, treats `Lost`/`Outdated` by reconfiguring and skipping the current present, skips `Timeout`/`Occluded`, and reports `Validation` as `GraphicsError::SurfaceStatus`.
- [x] Verify `SceneRenderer::present_frame_with_pipeline()` reuses the normal render-to-offscreen compiled-pipeline path, then presents the offscreen final color view through the viewport surface blit without calling `finish_viewport_frame()` or `read_texture_rgba()`.
- [x] Verify `present_frame_extract()` builds the normal submission context, prepares runtime sidebands, resolves history, validates viewport generation, records present submission via `record_present_submission()`, releases previous history, and updates stats without storing a `last_capture` CPU frame.
- [x] Extend focused `graphics_surface` tests for backend-independent behavior: surface helper clamping, format selection, advertised present-mode choice, missing-surface capture-counter preservation, and a source-level guard that the present path uses `SurfaceTexture::present()` without readback fallback calls.

**Testing Stage:**

- [x] Run `cargo test -p zircon_runtime --locked --verbose graphics_surface`.
- [x] Run `cargo test -p zircon_runtime --locked --verbose render_debugger` to ensure in-app RenderDoc capture state still works.
- [x] Run `cargo check -p zircon_runtime --locked` for target-gated surface code.
- [x] Debug rendering backend failures from lowest layer first: ABI target conversion, surface creation, surface configuration, acquire, render pass recording, present.
- [x] Acceptance evidence: source and test coverage verify the surface path reaches `SurfaceTexture::present()` without readback fallback; offscreen readback/capture tests remain green. Manual runtime-preview RenderDoc confirmation is still deferred to the app/manual milestone.

**2026-05-12 Milestone 3 Evidence:** The current tree already had the concrete WGPU present architecture under `viewport_surface.rs`, `SceneRenderer::present_frame_with_pipeline()`, and `present_frame_extract()`. This pass made the backend Win32 raw-handle creation target-gated, tightened present-mode selection to choose only advertised modes and reject empty advertised present-mode capability lists, added backend helper tests for zero-size clamping/format/present-mode selection, added a source-level no-readback guard for the present path, and asserted missing-surface present does not increment captured-frame counters. `cargo test -p zircon_runtime --locked --verbose graphics_surface` passed with `16 passed; 0 failed; 1248 filtered out` in the runtime test binary after the empty-present-mode fix. `cargo test -p zircon_runtime --locked --verbose render_debugger` passed with `11 passed; 0 failed; 1253 filtered out`. `cargo check -p zircon_runtime --locked` passed.

**Lightweight Checks:**

- [ ] During implementation, run only `cargo check -p zircon_runtime --locked` when unsafe surface code or lifetime changes produce type uncertainty.

## Milestone 4: Runtime Dynamic API Integration

**Goal:** Connect runtime ABI calls to render framework surface binding and presentation.

**In-scope behaviors:** ABI validation, session lookup, viewport conversion, surface target conversion, present frame request path, clear diagnostics for unsupported surface kinds, existing `capture_frame()` preserved.

**Dependencies:** Milestones 1 through 3.

**Implementation Slices:**

- [x] Add `zircon_runtime/src/dynamic_api/surface.rs` with conversion functions:

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

- [x] Modify `zircon_runtime/src/dynamic_api/session.rs` to add unsafe extern functions `bind_viewport_surface`, `unbind_viewport_surface`, and `present_viewport` that call `with_session()`.
- [x] Add methods to `RuntimeDynamicSession`:

```rust
fn bind_viewport_surface(
    &mut self,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus;

fn unbind_viewport_surface(&mut self, viewport: ZrRuntimeViewportHandle) -> ZrStatus;

fn present_viewport(&mut self, request: ZrRuntimeFrameRequestV1) -> ZrStatus;
```

- [x] In `present_viewport`, build the same scene extract currently used by `capture_frame()` and call the new render framework `present_viewport()` instead of `submit_frame_extract()` + `capture_frame()`.
- [x] Modify `zircon_runtime/src/dynamic_api/exports.rs` to populate the new optional function pointers.
- [x] Modify `zircon_runtime/src/dynamic_api/tests.rs` to assert the API exposes the new optional functions and unsupported surface target returns `ZrStatusCode::InvalidArgument` with diagnostics.
- [x] Add a dynamic API test where `bind_viewport_surface` on an invalid session returns `NotFound` and does not panic.

**Testing Stage:**

- [x] Run `cargo test -p zircon_runtime --locked --verbose dynamic_api`.
- [x] Run `cargo test -p zircon_runtime --locked --verbose graphics_surface`.
- [x] Debug correction loop covers status conversion, session lookup, render framework method dispatch, and fallback capture compatibility.
- [x] Acceptance evidence: dynamic API reports surface functions as available and preserves existing `capture_frame()` behavior.

**2026-05-12 Milestone 4 Evidence:** Runtime dynamic surface descriptor conversion now lives in `zircon_runtime/src/dynamic_api/surface.rs`, `session.rs` delegates bind conversion through that module while preserving pre-session validation order, and `zircon_runtime/src/dynamic_api/tests.rs` covers target ABI rejection, invalid-session bind with a valid Win32 descriptor, and `capture_frame()` wrong-ABI/unknown-viewport pre-session behavior. `cargo test -p zircon_runtime --locked --verbose dynamic_api` passed with `20 passed; 0 failed; 1248 filtered out` in the runtime test binary. `cargo test -p zircon_runtime --locked --verbose graphics_surface` passed with `16 passed; 0 failed; 1252 filtered out`.

**Lightweight Checks:**

- [ ] Use `cargo check -p zircon_runtime --locked` after function pointer and ABI export changes.

## Milestone 5: App Runtime Preview Surface Present Switch

**Goal:** Use the new runtime API from the real runtime preview window and make ordinary RenderDoc capture possible.

**In-scope behaviors:** Bind surface after window creation, rebind or resize on `SurfaceResized`, present on redraw without softbuffer when supported, fallback to softbuffer readback when binding or present returns unsupported, unbind on session/window teardown when available.

**Dependencies:** Milestones 1 through 4.

**Implementation Slices:**

- [x] Modify `zircon_app/src/entry/runtime_entry_app/runtime_entry_app.rs` or the existing struct declaration file to store:

```rust
pub(super) surface_present_enabled: bool,
pub(super) surface_present_failed: bool,
```

- [x] In `can_create_surfaces`, after `resize_viewport`, call `runtime_native_surface_target(window.as_ref())` and `RuntimeSession::bind_viewport_surface()` with `ZrRuntimeBindViewportSurfaceRequestV1::new(...)`.
- [x] Create `SoftbufferRuntimePresenter` only when native surface target extraction fails, bind returns `false`, or bind returns an unsupported/capability-denied status.
- [x] In `WindowEvent::SurfaceResized`, call both `resize_viewport()` and `bind_viewport_surface()` again when `surface_present_enabled` is true so the runtime backend reconfigures the swapchain size.
- [x] In `WindowEvent::RedrawRequested`, branch:

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

- [x] If surface present fails at runtime, initialize `SoftbufferRuntimePresenter` on demand and fall back to existing `capture_frame()` present in the same redraw branch.
- [x] Add diagnostic log entries for `runtime_surface_present_enabled`, `runtime_surface_present_fallback`, and `runtime_surface_present_failed` using existing diagnostic log utilities if available in `zircon_app`; otherwise keep errors scoped to existing runtime app error handling.
- [x] Ensure `about_to_wait()` continues requesting redraw so surface present is driven frame-by-frame.

**2026-05-12 Milestone 5 Implementation Note:** `RuntimeEntryApp` now tracks `surface_present_failed`, resizes the runtime viewport before initial native surface bind, gates redraw native presentation on `surface_present_enabled && !surface_present_failed`, logs enabled/fallback/failed transitions through `zircon_runtime::diagnostic_log`, disables/unbinds on bind or present failure, and creates the softbuffer fallback on demand in the same redraw path. `WindowEvent::SurfaceResized` now forwards `resize_viewport()` before rebinding the active surface-present path and preserves fallback presenter resize. Source-level `runtime_entry` tests were added under `zircon_app/src/entry/tests/mod.rs`; controller validation is pending, and manual runtime preview/RenderDoc acceptance remains pending.

**Testing Stage:**

- [x] Run `cargo test -p zircon_app --locked --verbose runtime_entry`.
- [x] Run `cargo check -p zircon_app --locked`.
- [ ] Run a manual Windows runtime preview smoke command from repository root:

```powershell
$env:WGPU_BACKEND='dx12'; $env:WGPU_DEBUG='1'; $env:WGPU_VALIDATION='1'; cargo run -p zircon_app --bin zircon_runtime --locked
```

- [ ] In RenderDoc, launch the runtime preview process with the same environment and verify the ordinary `Capture Frame` button captures a frame with a swapchain present boundary.
- [ ] Debug correction loop: if app falls back to softbuffer, inspect diagnostic status in this order: native handle extraction, ABI function availability, runtime surface bind, backend surface creation, present call.

**Lightweight Checks:**

- [x] Use `cargo check -p zircon_app --locked` after app state and event-loop changes.

**2026-05-12 Milestone 5 Evidence:** App runtime preview now prefers native surface present when the loaded runtime exposes the coherent optional ABI set and the winit window yields a Win32 surface descriptor. Initial binding happens after the runtime viewport is resized; resize events forward `resize_viewport()` before rebinding the active surface; redraw calls native `present_viewport()` only while `surface_present_enabled && !surface_present_failed`; bind or present failure disables/unbinds the native path and falls back to `capture_frame()` plus softbuffer presentation in the same redraw branch. Diagnostics cover `runtime_surface_present_enabled`, `runtime_surface_present_fallback`, and `runtime_surface_present_failed`. After spec review requested stronger source-level guards, `zircon_app/src/entry/tests/mod.rs` now checks initial resize-before-bind order, bind request construction, resize rebind order, same-branch present-failure fallback, failure marking, and teardown unbind tokens. The post-fix rerun of `cargo test -p zircon_app --locked --verbose runtime_entry` passed with `1 passed; 0 failed; 40 filtered out`, and `cargo check -p zircon_app --locked` passed. Manual Windows runtime preview and RenderDoc ordinary capture acceptance remain pending.

**2026-05-12 Milestone 5 Reviews:** Spec re-review approved Milestone 5 with no findings after the source guard strengthening. Code-quality review approved with no findings; residual risks are the still-pending manual Windows runtime-preview/RenderDoc acceptance, the accepted source-token nature of app flow tests, and the current `RuntimeSession::Drop` default viewport-handle assumption matching the app/runtime default viewport contract.

## Milestone 6: Documentation And Workspace Acceptance

**Goal:** Document the surface-present architecture, capture modes, fallback behavior, and validation evidence.

**In-scope behaviors:** Module docs with related-code headers, RenderDoc usage notes, validation records, offscreen fallback explanation, editor viewport scope statement.

**Dependencies:** Milestones 1 through 5.

**Implementation Slices:**

- [x] Update `docs/assets-and-rendering/render-framework-architecture.md` frontmatter to include new related files and describe `OffscreenReadback` versus `WindowSurface` viewport targets.
- [x] Update existing owner doc `docs/zircon_runtime/graphics/window-swapchain-present.md` instead of adding a duplicate `docs/zircon_runtime/graphics/surface-present.md`; the current implementation owner paths are `viewport_surface` and `present_frame_extract`, not the older planned `surface_target` directory. Required frontmatter coverage is represented as:

```markdown
---
related_code:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/surface.rs
plan_sources:
  - user: 2026-05-10 Add true wgpu Surface present path for RenderDoc ordinary capture.
tests:
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---
```

- [x] Add `docs/zircon_app/runtime-surface-present.md` with frontmatter covering app window binding and softbuffer fallback.
- [x] Document RenderDoc launch recipe:

```powershell
$env:WGPU_BACKEND='dx12'
$env:WGPU_DEBUG='1'
$env:WGPU_VALIDATION='1'
cargo run -p zircon_app --bin zircon_runtime --locked
```

- [x] Document acceptance distinction: ordinary RenderDoc capture is expected for runtime preview surface path; editor viewport remains on offscreen readback until the editor GPU embedding milestone.

**Testing Stage:**

- [x] Run `cargo test -p zircon_runtime_interface --locked --verbose`.
- [x] Run `cargo test -p zircon_runtime --locked --verbose` and record the unrelated blockers.
- [x] Run `cargo test -p zircon_app --locked --verbose`.
- [x] Run full workspace validation before final closeout after the unrelated runtime blockers are resolved, and record any remaining out-of-scope blocker:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1
```

- [x] Record validation evidence in final response and docs if failures required code changes.
- [ ] Acceptance evidence: runtime preview presents through `SurfaceTexture::present()`, softbuffer fallback still works when surface APIs are unavailable, tests pass, and docs map all affected code.

**2026-05-12 Milestone 6 Evidence:** Documentation now covers the app window-binding owner path, the runtime WGPU swapchain owner path, `OffscreenReadback` versus `WindowSurface`, RenderDoc launch commands, fallback diagnostics, and the editor-viewport scope boundary. `cargo test -p zircon_runtime_interface --locked --verbose` passed with `94 passed; 0 failed`. Scoped runtime validation passed through `cargo test -p zircon_runtime --locked --verbose graphics_surface`, `cargo test -p zircon_runtime --locked --verbose dynamic_api`, and `cargo test -p zircon_runtime --locked --verbose render_debugger`. `cargo test -p zircon_app --locked --verbose` passed after formatting with `41 passed; 0 failed`; the package's runtime-preview binary test and doc tests also ran zero tests successfully. The first broad `cargo test -p zircon_runtime --locked --verbose` run failed outside the surface-present scope with `1264 passed; 4 failed`: `ui::tests::component_catalog::material_editor_foundation_catalog_covers_planned_component_layers` missed `TreeView.expanded`, and three `tests::plugin_extensions::native_plugin_loader::*` cases found stale `zircon_plugins/Cargo.lock` entries under `--locked`.

**2026-05-12 Runtime Blocker Follow-up Evidence:** The runtime/UI/plugin blockers were resolved in their lowest shared layers: Material foundation metadata now includes `TreeView.expanded`, `zircon_plugins/Cargo.lock` was refreshed, default runtime-world render extraction no longer injects a non-empty effective VG payload, and frame submission preserves an explicitly authored empty `RenderVirtualGeometryExtract` as a `RenderPathClearOnly` frame instead of falling through to provider synthesis. Focused validation passed for `cargo test -p zircon_runtime --test virtual_geometry_stats_contract --locked --verbose`, `cargo test -p zircon_runtime --test virtual_geometry_execution_snapshot_contract --locked --verbose`, and `cargo test -p zircon_runtime --test m1_runtime_editor_boundary_contract --locked --verbose`. Broad runtime validation then passed with `cargo test -p zircon_runtime --locked --verbose`, including `1268 passed; 0 failed` in the runtime lib test binary plus integration/doc-test targets. The full workspace validator `./.opencode/skills/zircon-dev/scripts/validate-matrix.ps1 -VerboseOutput` passed the workspace build, then failed during `cargo test --workspace --locked --verbose --target-dir target/codex-shared-a` at `zircon_editor --lib`; the first concrete assertion is `tests::host::template_runtime::pane_body_documents::host_projection_carries_runtime_component_properties_and_routes` at `zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs:616`, where `NameField.text` is `Some("Name")` but the test expects `None`. Later `PoisonError` failures are cascade from that retained-host/template panic. That blocker overlaps the active retained UI cutover lane and remains out of scope for this runtime-surface milestone unless ownership is explicitly transferred. Manual Windows runtime preview and RenderDoc ordinary-capture acceptance remain pending because they require an interactive GPU/window capture session.

**2026-05-12 Milestone 6 Reviews:** Spec compliance review passed with no findings. Documentation/coordination quality review approved with no findings. Both reviews recorded the same residual risk for the runtime-surface lane: manual Windows runtime-preview/RenderDoc capture remains pending. Full workspace validation is no longer blocked by runtime/UI catalog or plugin-lockfile failures; it now stops in the retained editor/template projection lane described above.

**Lightweight Checks:**

- [ ] No additional lightweight check is needed after docs-only changes; defer full verification to this milestone testing stage.

## Risks And Guardrails

- Unsafe surface creation must stay in `zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs`; no unsafe raw window handle code belongs in editor UI or app event-loop files beyond handle extraction.
- `zircon_runtime_interface` must remain C-safe. Do not expose Rust enums, `raw_window_handle` types, references, `Arc`, or trait objects across the dynamic API.
- Surface present must not remove `capture_frame()`. Tests, headless runs, unsupported platforms, and editor viewport still need readback fallback.
- If wgpu surface lifetimes require owning handles differently than the ABI descriptor allows, stop and redesign the surface ownership boundary rather than storing borrowed window references in runtime.
- Do not expand `zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs` or other large editor hotspots for this milestone.

## Plan Self-Review

- Spec coverage: runtime ordinary RenderDoc capture is covered by Milestones 3 and 5; fallback and in-app capture preservation are covered by Milestones 1, 2, and 6; docs are covered by Milestone 6.
- Placeholder scan: the plan contains concrete files, method names, commands, tests, and acceptance evidence; it avoids unspecified implementation slots.
- Type consistency: ABI types use `ZrRuntime*V1`; framework types use `Render*`; backend types remain crate-private under `graphics/backend`.
