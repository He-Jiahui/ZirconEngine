# Plugins14 Demand-Driven Overlay Frame Optimization Record

- Date: 2026-08-19
- Owner: `plugins14-arc-assets-demand-overlay-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md`, NNAV-P1-043
- Status: implementation complete; combined managed validation pending

## Problem

The mirrored overlay event already tracked reader count in
`NavigationDebugCapture`, and agent path diagnostics honored that flag. The
runtime system nevertheless rebuilt and sent a full navmesh overlay every
frame, cloning all debug triangles and off-mesh links even when the mirrored
event had no readers.

## Change

- A single helper checks `NavigationDebugCapture` before any overlay work.
- With no reader, the system emits the normal agent tick report but does not
  clone the report for overlay ownership, project navmesh triangles/links, or
  send a `NavigationOverlayFrame`.
- With at least one reader, payload fields, owner generation, event ordering,
  and mirrored-event behavior are unchanged.

## Deterministic Performance Evidence

| Workload | Before | After with no readers | Reduction |
|---|---:|---:|---:|
| One frame over 32,768 navmesh triangles | 32,768 triangle projections | 0 | 100% |
| Overlay frame allocations | 1 full frame | 0 | 100% |
| Overlay events sent | 1 | 0 | 100% |

## Acceptance

- `overlay_frame_is_projected_only_while_debug_capture_is_enabled` verifies
  disabled and enabled behavior against the same manager and World.
- Existing runtime mirror tests continue to verify that reader-count changes
  toggle `NavigationDebugCapture`.
- `demand_driven_overlay_frame_release_benchmark_evidence` compares 21 paired,
  alternating release samples over 32,768 triangles and four frames, then
  computes nearest-rank P50/P95.
- Timing gate: disabled overlay P95 must be no more than 1% of unconditional
  full-frame projection P95.
- Exact-file Rustfmt, scoped source assertions, and `git diff --check`: passed.
- Cargo regression and release P50/P95: pending the same batched Windows
  coordinator validation as Arc loaded-navmesh snapshots.

## Remaining Scope

When readers are present, the runtime still rebuilds the complete overlay on
every frame, and the editor later expands every triangle into three lines.
Generation-cached edges, viewport/selection/frustum/LOD filters, incremental
transport, and explicit items/bytes/time budgets remain open under
NNAV-P1-043.
