---
related_code:
  - dev/bevy/crates/bevy_render/src/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - docs/zircon_runtime/core/framework/render/camera.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
implementation_files:
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - docs/zircon_runtime/core/framework/render/camera.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
plan_sources:
  - user: 2026-05-17 continue Bevy-level rendering completion after M2C-A
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras
doc_type: milestone-detail
---

# Render Camera Ordering M2D Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Zircon a Bevy-style neutral camera ordering contract before multi-camera split-screen, render-to-texture, and per-camera core pipeline scheduling expand beyond the current single-frame extract path.

**Architecture:** Keep camera ordering in `zircon_runtime::core::framework::render`, next to the camera contract. Do not touch editor authoring, scene serialization cutover, or concrete texture writeback in this slice.

**Tech Stack:** Rust 2021, existing `ViewportCameraSnapshot`, local Bevy source under `dev/bevy/crates/bevy_render/src/camera.rs`.

---

## Reference Evidence

- Bevy registers `sort_cameras` in the render app create-views set at `dev/bevy/crates/bevy_render/src/camera.rs:108`.
- Bevy `SortedCameras` stores cameras sorted by order at `dev/bevy/crates/bevy_render/src/camera.rs:663`.
- Bevy `sort_cameras` sorts by `(order, target)`, detects duplicate `(order, target)` ambiguities, and assigns `sorted_camera_index_for_target` per `(target, hdr)` at `dev/bevy/crates/bevy_render/src/camera.rs:674-722`.

## Implementation Slices

- [x] Add `camera_ordering` under `zircon_runtime/src/core/framework/render`.
- [x] Define `RenderCameraOrderInput`, `SortedRenderCamera`, `RenderCameraTargetOrderKey`, `RenderCameraOrderAmbiguity`, and `RenderCameraOrderReport`.
- [x] Implement `sort_render_cameras(...)` so it:
  - skips inactive cameras;
  - sorts active cameras by `order`, then normalized target key, then deterministic entity tie-break;
  - reports duplicate active `(order, target)` groups;
  - assigns per-`(target, hdr)` `sorted_camera_index_for_target`.
- [x] Add framework tests for ordering, target/HDR index assignment, inactive camera filtering, and ambiguity reporting.
- [x] Update camera module docs, the rendering capability matrix, and the active coordination note.

## Testing Stage

- Run `rustfmt --edition 2021 --check` on the touched Rust files.
- Run `cargo test -p zircon_runtime render_camera_ordering --locked --jobs 1 --message-format short --color never` with a dedicated external `CARGO_TARGET_DIR`.
- Run `cargo check -p zircon_runtime --lib --locked --message-format short --color never` with the same target directory if the focused test passes.
- Run `git diff --check` on the touched code/docs/session files.

## Acceptance Evidence

- `rustfmt --edition 2021 --check zircon_runtime\src\core\framework\render\camera_ordering.rs zircon_runtime\src\core\framework\render\mod.rs zircon_runtime\src\core\framework\tests.rs` passed on 2026-05-17.
- Windows default-feature focused Cargo validation was attempted with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2d`, but dependency compilation failed before Zircon code at `wgpu-hal 29.0.3` DX12/windows type mismatches. The same failure reproduced with `--no-default-features --features core-min`; it is not M2D source evidence.
- WSL/Linux focused validation passed on 2026-05-17 with `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-render-camera-m2d-wsl`: `cargo test -p zircon_runtime --lib render_camera_ordering --locked --jobs 1 --message-format short --color never` ran 2 tests, 2 passed, 0 failed, 1520 filtered out.
- WSL/Linux lightweight type check passed on 2026-05-17 with the same target directory: `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`. It emitted only existing unused-function warnings outside the M2D files.
- `git diff --check` on tracked touched code/docs/session files passed on 2026-05-17 with line-ending warnings only; direct trailing-whitespace scan on the untracked M2D plan and new `camera_ordering.rs` file found no trailing whitespace.
