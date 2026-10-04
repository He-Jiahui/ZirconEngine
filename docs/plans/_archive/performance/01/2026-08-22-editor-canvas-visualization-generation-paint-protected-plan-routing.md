---
title: Editor canvas visualization generation and paint protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-canvas-visualization-generation-paint-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor canvas visualization generation/projection/host paint (sample_grid, timeline_strip, weight_heatmap)`
- 56/56 Rust files source-reviewed. Interaction still rebuilds/rehashes full generations; sample and
  timeline expand keys/points into many quads, heatmap miss is O(cells * sources) and paint is one
  command per cell. M1 removes collapsed-frame text measurements 4 -> 0, owned label strings 2 -> 0
  and painter dispatches 3 -> 0 (focused GREEN 2/2; owned contracts GREEN 52/52). M0/M2-M6
  instrumentation, typed retained generations, visible budgets, heat algorithm, batch/display-list
  and profile/power acceptance remain pending.

Do not add these folders to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to MVP animation/visualization surfaces. Record parse/hash/artifact/command/batch counts,
heat pair/exp evaluations, cache/lock behavior, CPU/allocation/RSS/input latency/context switches,
RenderDoc draw/GPU evidence and WPR power/energy.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of attribute-driven whole-generation reconstruction and process-global presentation
caches after typed source/view consumers migrate.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own ordered playhead/selection/drag receipts and latest-wins pointer coalescing without reordering
interaction edges.

## `docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`

Own typed sample/key/source identity, immutable source generation, input budgets and schema diagnostics.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own per-view retained visualization generations, visible queries, static/dynamic invalidation and
generation-qualified application of prepared artifacts.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own batched line/point/key/heat-cell display-list primitives and shared paint/hit identity.

## `docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md`

Route timeline/sample-grid product source identity, visible-range querying and curve/key invalidation
to the canonical animation authoring plan; do not create a second timeline model in performance work.

## Acceptance handoff

The handoff requires 56/56 post-change fingerprints, managed Rust behavior tests, the scale/input
matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc draw/GPU plus pixel/text/hit parity,
milestone commit and quantified WeCom notification. Protected ledgers remain unchanged until then.
