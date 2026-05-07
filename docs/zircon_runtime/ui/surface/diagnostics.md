---
related_code:
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-debug-reflector-full-closure-design.md
  - docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md
  - user: 2026-05-06 continue UI Debug Reflector full closure milestone
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - tests/acceptance/ui-debug-reflector-full-closure.md
doc_type: module-detail
---

# Runtime UI Surface Diagnostics

Runtime UI surface diagnostics generate the shared debug snapshot consumed by the editor Debug Reflector. The snapshot source is `UiSurfaceFrame`, which already contains arranged nodes, render extract, hit grid, focus state, and rebuild counters.

## Snapshot Contract

`zircon_runtime_interface::ui::surface::UiSurfaceDebugSnapshot` is the neutral serde contract. It carries schema version/capture context, widget reflector nodes, render command records, material batches, `UiRenderDebugSnapshot.render_batches`, hit-grid cell records, pick hit-test dumps, sampled overdraw cells, invalidation and damage reports, event records, and overlay primitives.

`render_batches` is the replay-friendly render slice for M7 tools. It carries batch/debug rows, cache status, renderer parity rows, and `UiRenderVisualizerSnapshot` paint rows, batch groups, overlays, overdraw regions, resource bindings, text backend/glyph/decorator counters, and cache reuse/rebuild stats. Editor Debug Reflector reads this packet directly; renderer-specific code does not need to expose private draw commands.

`UiDebugOverlayPrimitiveKind` covers both surface diagnostics and render visualizer replay. Besides selected frames, clip frames, hit cells, hit paths, rejected bounds, overdraw cells, material batch bounds, and damage regions, it now includes render wireframes, text glyph bounds, text baselines, and resource atlas bounds.

Stable hit-test reject reasons live in `UiHitTestRejectReason`, with a separate human-readable message. Editor tests should assert reason codes instead of string-matching messages.

## Runtime Generation

`debug_surface_frame_with_options(...)` builds the baseline snapshot. `debug_surface_frame_for_selection(...)` adds selected-node capture and selection overlay primitives. `debug_surface_frame_for_pick(...)` runs query-aware frame hit testing, stores the pick dump, selects the top hit, and emits hit-path/rejected-bounds overlay primitives.

`UiSurface::debug_snapshot_for_selection(...)`, `UiSurface::debug_snapshot_for_pick(...)`, and `UiSurface::debug_snapshot_json(...)` are convenience methods for callers that already own a retained surface. JSON export remains payload-only; editor code owns paths and import/export UI.

## Current Limits

Backend render counters are optional until host render backends provide measured draw-call and pipeline data. Runtime damage reporting is currently default/empty; editor host damage integration should populate `UiDamageDebugReport` later without changing the snapshot authority model.

## Validation

M7 diagnostics validation on 2026-05-07 used `E:\zircon-build\targets-ui-m7`: `cargo check -p zircon_runtime_interface --tests`, `cargo test -p zircon_runtime --lib diagnostics`, and `cargo test -p zircon_runtime --lib hit_grid`. These passed with existing runtime warning noise.
