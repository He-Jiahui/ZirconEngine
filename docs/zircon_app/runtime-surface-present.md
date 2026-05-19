---
related_code:
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/core/framework/window/resize_constraints.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
implementation_files:
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/runtime_presenter.rs
plan_sources:
  - user: 2026-05-10 switch runtime preview to real wgpu surface present for ordinary RenderDoc capture
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - docs/superpowers/plans/2026-05-10-runtime-surface-present.md
tests:
  - zircon_app/src/entry/runtime_entry_app/window_attributes.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - cargo test -p zircon_app --locked --verbose runtime_entry
  - cargo test -p zircon_app runtime_entry_maps_platform_event_loop_policy_to_winit_control_flow --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
  - cargo test -p zircon_app --locked --verbose
  - cargo check -p zircon_app --locked
doc_type: module-detail
---

# Runtime Surface Present

`zircon_app` owns the runtime-preview process entry, winit window, native handle extraction, and softbuffer fallback. It does not create WGPU devices, surfaces, swapchains, render pipelines, or scene renderer state. Those stay in `zircon_runtime` behind the dynamic runtime API.

## Startup Bind

Before creating the dynamic session, the runtime runner strips diagnostic log startup arguments and then accepts `--runtime-session-profile <runtime|editor|dev|minimal|headless>` or `--runtime-session-profile=dev`. The selected value is forwarded as `ZrRuntimeSessionConfigV1.profile` through `RuntimeSession::create_with_profile(...)`. `-h` and `--help` return a startup help page that lists the profile names, process log controls, and dynamic runtime library override without opening the winit window or loading the runtime library. In the current Bevy-style dev profile path, `dev` enables runtime-owned diagnostic-store log cadence from `zircon_runtime`; the app runner only forwards the startup policy and does not inspect or emit runtime diagnostics itself.

The runtime app creates the preview window in `can_create_surfaces`, reads the window surface size, stores a nonzero `ZrRuntimeViewportSizeV1`, and calls `resize_viewport()` before attempting any native surface bind. This ordering keeps runtime viewport metadata aligned with the first WGPU surface configuration.

Window creation now starts from the neutral runtime `WindowDescriptor` instead of hard-coded app literals. `runtime_entry_app/window_attributes.rs` translates the descriptor into winit `WindowAttributes`: title, physical surface size, resize constraints, resizable/decorated/visible/focused flags, explicit physical position, and current fullscreen intent. `WindowPosition::Centered` remains a later host-placement step because winit needs monitor information at creation time; `WindowMode::Fullscreen` currently maps to borderless fullscreen until the host owns explicit monitor/video-mode selection. This keeps the Bevy-style split intact: `zircon_runtime::core::framework::window` owns the window vocabulary, while `zircon_app` owns the concrete winit representation.

After resize, `runtime_native_surface_target(window)` converts the winit window and display handles into a `ZrRuntimeNativeSurfaceTargetV1`. The current app-side extractor emits only Win32 descriptors. If extraction fails, or if the loaded runtime does not expose a coherent optional bind/unbind/present ABI set, the app skips native present and creates `SoftbufferRuntimePresenter` for the existing CPU readback path.

When extraction succeeds, `RuntimeSession::bind_viewport_surface()` sends a `ZrRuntimeBindViewportSurfaceRequestV1::new(...)` request through the optional dynamic ABI. A successful bind calls `enable_surface_present()` and logs `runtime_surface_present_enabled`. An unavailable bind logs `runtime_surface_present_fallback`. A bind error logs `runtime_surface_present_failed`, unbinds best-effort when a bind was attempted, and then uses the fallback presenter.

## Redraw Flow

`WindowEvent::RedrawRequested` first checks `surface_present_enabled && !surface_present_failed`. While that gate is true, redraw calls `RuntimeSession::present_viewport()` and returns immediately on `Ok(true)`. This is the app-side entry to the runtime WGPU path that eventually calls `SurfaceTexture::present()`.

If `present_viewport()` returns `Ok(false)` or an error, the app marks `surface_present_failed`, logs `runtime_surface_present_failed`, disables/unbinds the native path, creates the softbuffer presenter on demand, and continues through the existing fallback branch in the same redraw event. The fallback branch calls `RuntimeSession::capture_frame()` and passes the CPU RGBA frame to `SoftbufferRuntimePresenter::present()`.

`about_to_wait()` applies the current platform event-loop policy before runtime ticks and redraw scheduling. The runtime-preview host maps `EventLoopPolicy::Game` and `EventLoopPolicy::Continuous` to winit `ControlFlow::Poll`, and maps `DesktopApp`, `Mobile`, and `Headless` to `ControlFlow::Wait`. The preview app still calls `request_redraw()` on the window each loop after the runtime tick and host-request drain so both native surface present and softbuffer fallback remain frame-driven.

## Resize And Teardown

`WindowEvent::SurfaceResized` always forwards the clamped size through `resize_viewport()`. If native present is active and has not failed, the app then calls `bind_current_window_surface()` again so the runtime backend can reconfigure the viewport surface for the new size. If a softbuffer presenter exists, it is resized as before.

`RuntimeEntryApp::drop()` calls `disable_surface_present()`, which unbinds the viewport surface when native present is enabled or was attempted. `RuntimeSession::drop()` also performs a best-effort unbind for the current default runtime viewport before destroying the session. That duplicate cleanup is intentionally harmless because the ABI unbind path is optional and best-effort.

## RenderDoc Launch

For manual Windows validation, launch the runtime preview from RenderDoc or from a shell with the same environment:

```powershell
$env:WGPU_BACKEND='dx12'
$env:WGPU_DEBUG='1'
$env:WGPU_VALIDATION='1'
cargo run -p zircon_app --bin zircon_runtime --locked
```

If testing with explicit features instead of defaults, use `cargo run -p zircon_app --no-default-features --features target-client --bin zircon_runtime --locked`.

RenderDoc ordinary `Capture Frame` should show the runtime process as presenting and the captured frame should end in a swapchain present after `zircon-present-blit-pass`. If the app falls back to softbuffer, inspect diagnostics in this order: native handle extraction, optional ABI availability, runtime surface bind, backend surface creation/configuration, and present call status.

## Scope

This path is for runtime preview. Editor viewport embedding remains on offscreen readback until a separate editor GPU embedding milestone. The fallback path is not legacy dead code; it remains required for unsupported native surfaces, dynamic runtimes without the optional ABI fields, headless/test workflows, and editor viewport imports.

## Validation

The 2026-05-12 app validation ran `cargo test -p zircon_app --locked --verbose runtime_entry`, `cargo check -p zircon_app --locked`, `cargo fmt -p zircon_app --check`, and the full package command `cargo test -p zircon_app --locked --verbose`. The final full package run passed with `41 passed; 0 failed`; the runtime-preview binary test target and doc tests both ran zero tests successfully. Manual RenderDoc validation remains separate because it needs an interactive Windows GPU/window capture session.

Workspace validation has also exercised this app path through `./.opencode/skills/zircon-dev/scripts/validate-matrix.ps1 -VerboseOutput`: the workspace build phase passed, and the later test phase reached `zircon_editor --lib` before failing in retained host template projection. No app-side runtime-surface regression was identified in that validator run.

The runtime session profile forwarding slice adds a focused app entry source guard plus parser unit coverage for the new startup argument and help output. Current slice validation is limited to Rust formatting and source hygiene while concurrent Cargo/rustc jobs are active; package-level `zircon_app` validation must be rerun at the milestone testing stage.
