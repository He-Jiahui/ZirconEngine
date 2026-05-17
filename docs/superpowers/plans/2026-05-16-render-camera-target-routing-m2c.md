# Render Camera Target Routing M2C Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Zircon camera render targets affect concrete graphics submission without silently falling back to the primary viewport.

**Architecture:** Keep target declarations in `zircon_runtime::core::framework::render::camera`; keep concrete target routing in `zircon_runtime::graphics::runtime::render_framework`. `Headless` targets resolve to an offscreen submission size immediately. `Texture` targets are rejected with a structured unsupported-capability error until asset-backed GPU texture residency is ready.

**Tech Stack:** Rust 2021, WGPU render framework, Bevy reference source under `dev/bevy/crates/bevy_camera` and `dev/bevy/crates/bevy_render`.

---

## Reference Evidence

- Bevy separates viewport and target sizes in `dev/bevy/crates/bevy_camera/src/camera.rs`.
- Bevy extraction carries both viewport and target size in `dev/bevy/crates/bevy_render/src/camera.rs`.
- Bevy reports missing image or texture-view render targets with `MissingRenderTargetInfoError` instead of rendering to a different target.

## Implementation Slices

- [x] Add focused graphics tests in `zircon_runtime/src/graphics/tests/surface_targets.rs`:
  - `Headless { size }` controls the captured offscreen frame size even when the created viewport has a different size.
  - `Texture(handle)` returns `RenderFrameworkError::UnsupportedCapability { capability: "camera texture render target" }` and does not capture a frame.
- [x] Resolve the submission target size during `build_frame_submission_context`:
  - `PrimarySurface` uses the viewport record size.
  - `Headless { size }` uses the clamped target size.
  - `Texture(_)` returns the unsupported-capability error.
- [x] Apply the resolved target size back to the extract before visibility, history, and runtime-frame construction so camera aspect ratio follows the concrete target.
- [x] Keep `present_frame_extract` primary-surface-only for this milestone; `Headless` and `Texture` presentation must not silently blit to a bound surface.
- [x] Update `docs/zircon_runtime/core/framework/render/camera.md`, `docs/assets-and-rendering/bevy-rendering-capability-matrix.md`, and the active session note.

## Testing Stage

- Run `rustfmt --edition 2021 --check` on touched Rust files.
- Run `cargo test -p zircon_runtime camera_target --locked --jobs 1 --message-format short --color never` with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c`.
- Run `cargo check -p zircon_runtime --lib --locked --message-format short --color never` with the same target dir.
- If the focused test name changes, use the exact new test-name filter and record it in the docs.

## Acceptance Evidence

- `rustfmt --edition 2021 --check` on the touched Rust files passed on 2026-05-16.
- `git diff --check` on touched code/docs/session files passed on 2026-05-16 with line-ending warnings only.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c cargo test -p zircon_runtime camera_target --locked --jobs 1 --message-format short --color never` passed on 2026-05-16: 3 focused tests passed, 0 failed, 1500 filtered out.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed on 2026-05-16.
