# Runtime Gizmos M3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the M3 runtime-owned gizmo API that can emit immediate and retained debug geometry, configure rendering behavior, and adapt to the current render overlay DTOs without cutting over editor transform handles yet.

**Architecture:** `zircon_runtime::core::framework::gizmos` owns neutral gizmo commands, buffers, configs, and retained gizmo data. Existing `zircon_runtime::core::framework::render::RenderOverlayExtract` remains the output boundary for graphics/editor consumers in M3, with a one-way adapter from gizmo commands into overlay DTOs. Transform gizmo interaction and editor viewport cutover remain M4/M9 work.

**Tech Stack:** Rust 2021, `zircon_runtime::core::{framework,math}`, existing `RenderOverlayExtract` DTOs, milestone-first validation with Cargo.

---

## Current Baseline

- M2 picking validation passed with `cargo test -p zircon_runtime picking --locked --target-dir "E:\cargo-targets\zircon-runtime-picking-validation" --message-format short --color never`: 12 focused picking tests passed, 0 failed, 1059 filtered.
- Runtime render overlay DTOs already exist in `zircon_runtime/src/core/framework/render/overlay.rs`: grid, handles, scene gizmos, selection, lines, wire shapes, icons, and pick shapes.
- Editor viewport code currently builds scene gizmos and handles as editor-owned overlays. M3 does not delete those paths.
- Reference evidence used for this plan:
  - `dev/bevy/crates/bevy_gizmos/src/lib.rs` for plugin/config group shape and immediate/retained split.
  - `dev/bevy/crates/bevy_gizmos/src/gizmos.rs` for `GizmoBuffer`, clear contexts, line/ray/strip/cube APIs, and disabled-buffer behavior.
  - `dev/bevy/crates/bevy_gizmos/src/config.rs` for `enabled`, line config, depth bias, and render layers.
  - `dev/bevy/crates/bevy_gizmos/src/retained.rs` for retained `GizmoAsset`/`Gizmo` split.
  - `dev/Fyrox/editor/src/interaction/gizmo/move_gizmo.rs` for editor gizmos as consumers of scene/editor state rather than runtime world serialization.
  - `dev/Fyrox/editor/src/scene_viewer/gizmo.rs` for keeping viewport-orientation gizmos isolated from core scene authority.

## File Structure

- Create: `zircon_runtime/src/core/framework/gizmos/mod.rs`
  - Structural module declarations and public re-exports only.
- Create: `zircon_runtime/src/core/framework/gizmos/command.rs`
  - `GizmoCommand` enum and primitive command payloads.
- Create: `zircon_runtime/src/core/framework/gizmos/buffer.rs`
  - `GizmoBuffer`, clear behavior, immediate primitive helpers, and command accessors.
- Create: `zircon_runtime/src/core/framework/gizmos/config.rs`
  - `GizmoConfig`, `GizmoLineConfig`, `GizmoRenderLayer`, `GizmoColorPolicy`, `GizmoScreenScalePolicy`, and `GizmoConfigGroupId`.
- Create: `zircon_runtime/src/core/framework/gizmos/retained.rs`
  - `GizmoAsset` and `RetainedGizmo` data-only retained-gizmo contracts.
- Create: `zircon_runtime/src/core/framework/gizmos/extract.rs`
  - One-way adapter from `GizmoBuffer` / `GizmoAsset` commands to `RenderOverlayExtract` scene-gizmo line and wire-shape DTOs.
- Modify: `zircon_runtime/src/core/framework/mod.rs`
  - Add `pub mod gizmos;` near other shared framework contracts.
- Modify: `zircon_runtime/src/tests/mod.rs`
  - Add `mod gizmos;`.
- Create: `zircon_runtime/src/tests/gizmos/mod.rs`
  - Focused M3 runtime gizmo tests.
- Create: `docs/zircon_runtime/core/framework/gizmos.md`
  - Module doc with `related_code`, `implementation_files`, plan sources, and test evidence.

## Milestone M3.1: Framework Gizmo Contracts

Status: Completed on 2026-05-08 with focused locked `zircon_runtime` gizmo validation.

- Goal: Establish the neutral gizmo data model without changing editor or graphics callers.
- In-scope behaviors: command variants, config fields, disabled-buffer no-op behavior, immediate buffer construction, retained data containers.
- Dependencies: `core::math::{Real, Transform, Vec3, Vec4}` and existing `core::framework::render` overlay DTOs.

### Implementation Slices

- [x] Add `framework::gizmos` module wiring in `zircon_runtime/src/core/framework/mod.rs` and `zircon_runtime/src/core/framework/gizmos/mod.rs`.
- [x] Define `GizmoCommand` variants for `Line`, `Ray`, `LineStrip`, `Rect`, `Circle`, `Sphere`, `Cube`, `Aabb`, and `Axis` in `command.rs`.
- [x] Define `GizmoConfig`, `GizmoLineConfig`, `GizmoRenderLayer`, `GizmoColorPolicy`, `GizmoScreenScalePolicy`, and `GizmoConfigGroupId` in `config.rs`.
- [x] Implement `GizmoBuffer` in `buffer.rs` with `new`, `with_config`, `clear`, `is_empty`, `commands`, `push_command`, and immediate helpers matching the M3 command set.
- [x] Ensure disabled buffers ignore new commands and retain no stale command growth.
- [x] Define `GizmoAsset` and `RetainedGizmo` in `retained.rs` as data-only contracts that can reuse `GizmoBuffer` content.
- [x] Add unit-test code in `zircon_runtime/src/tests/gizmos/mod.rs` for buffer push/clear, disabled no-op, config defaults, retained asset reuse, and command ordering.

### Testing Stage

- Run `rustfmt --edition 2021 --check` on all new gizmo files and touched module files.
- Run `cargo test -p zircon_runtime gizmo --locked --target-dir "E:\cargo-targets\zircon-runtime-gizmos-m3" --message-format short --color never`.
- If a higher-layer compile failure appears before gizmo tests run, apply support-first debugging: identify whether the failure is in `framework::gizmos`; if not, record it as external unless it is a direct shared-layer break caused by M3 files.

### Exit Evidence

- All M3.1 focused gizmo tests pass, or the session note records the exact external compile blocker.
- `docs/zircon_runtime/core/framework/gizmos.md` explains contracts, reference evidence, intentional divergence, and validation.

## Milestone M3.2: Overlay Extract Adapter

Status: Completed on 2026-05-08 with focused locked `zircon_runtime` gizmo validation.

- Goal: Prove runtime gizmo commands can feed the existing render overlay packet without replacing it.
- In-scope behaviors: command-to-overlay conversion, layer/depth metadata preservation where existing DTOs can carry it, and snapshot stability for supported primitives.
- Dependencies: M3.1 command/config model and current `RenderOverlayExtract`.

### Implementation Slices

- [x] Add `GizmoOverlayExtractRequest` or an equivalent narrow input struct in `extract.rs` with owner, kind, selected state, buffer, and retained gizmo inputs.
- [x] Convert `Line`, `Ray`, and `LineStrip` commands into `OverlayLineSegment` records.
- [x] Convert `Sphere`, `Cube`, `Aabb`, `Rect`, `Circle`, and `Axis` commands into the closest existing `OverlayWireShape` or line-segment representation without inventing a new renderer path.
- [x] Preserve `GizmoConfig.enabled` as a conversion gate.
- [x] Preserve render-layer and depth-bias values in gizmo-side config and document that current `RenderOverlayExtract` cannot yet carry every value until graphics overlay DTOs grow in a later rendering milestone.
- [x] Add unit-test code for line/ray/strip conversion, disabled conversion, retained asset extraction, and mixed immediate/retained command order.

### Testing Stage

- Run `rustfmt --edition 2021 --check` on all M3 files.
- Run `cargo test -p zircon_runtime gizmo --locked --target-dir "E:\cargo-targets\zircon-runtime-gizmos-m3" --message-format short --color never`.
- Run `git diff --check -- Cargo.lock docs/zircon_runtime/core/framework/gizmos.md zircon_runtime/src/core/framework/mod.rs zircon_runtime/src/core/framework/gizmos zircon_runtime/src/tests/mod.rs zircon_runtime/src/tests/gizmos`.

### Exit Evidence

- Focused gizmo tests cover all M3 command variants and adapter paths.
- Docs record the accepted divergence: runtime gizmos are plain DTOs and pure helpers, not Bevy ECS system params, and render overlay remains an output adapter until a later renderer milestone.

## Out Of Scope

- No transform gizmo interaction state, axis drag math, snapping, cursor confinement, or undo integration. That is M4.
- No editor viewport cutover from existing handle overlays to runtime gizmos. That is M9.
- No GPU line mesh renderer rewrite or render graph pass changes. M3 only adapts to existing overlay DTOs.
- No compatibility shim for old editor route APIs; existing editor code remains a consumer until the planned cutover milestone.
