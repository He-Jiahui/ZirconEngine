---
title: Editor native pointer drag-session capture and damage protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-pointer-drag-session-capture-damage-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the matching portions of the obsolete 2026-07-17 native-pointer drag/resize and retained-tab
drag coverage with one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/native_pointer/{drag_resize,tab_drag_damage}.rs + matching folders`
- 33/33 current Rust files source-reviewed. Threshold, committed-frame routing, retained shell hit
  surface and change-gated resize foundations are retained. M0 scalar move state, duplicate-point
  gates and borrowed model rows are applied and statically GREEN. Pending M1-M4 and M0 dynamic
  acceptance: fully allocation-free typed/indexed routes and drag/resize generations, one atomic drop
  transaction, exact typed multi-region damage, managed scale/WPR/power/RenderDoc acceptance.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP docking and pointer-to-present latency. Record pointer rate, drag-state String
allocations/bytes, state clones/publications, route hits, target transitions, rows visited/cloned,
layout/chrome/context/model builds, command-lock wait/hold, resize patches/recomputes, damage useful/
union/submitted area, CPU p50/p95/p99, WPR scheduling/power and exact source/workload fingerprints.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of the flat string drag-state authority, duplicate release hit/model resolver and
non-atomic multi-command tab-drop paths after typed generations and transactions are ready.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own typed ordered drag/resize begin, move, up and cancel receipts; latest-wins motion coalescing;
pointer capture; retained previous/current route transitions; generation drift policy.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own `DragSessionGeneration`, indexed docking owner/target projection, committed layout generation,
single final-hit reuse and atomic `ApplyTabDrop` transaction.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own stable typed route identities and the allocation-free hit-to-target transition contract. Route
keys are diagnostic/persistence output, not the hot-path identity authority.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Own compact scalar session patches and one change-proportional publication. Static drag payload must
not be rebound or reprojected on pointer motion.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable retained pointer-path transition, capture and reason-coded damage contracts shared with
editor UI without editor-specific string group semantics.

## Related performance records

- `2026-08-22-editor-drag-world-surface-generation-architecture-review.md` owns drag-overlay static/
  dynamic projection separation and world-surface generation.
- `2026-08-23-editor-native-pointer-effect-damage-projection-architecture-review.md` owns typed action
  invalidation effects and retained owner/overlap projection.
- `2026-08-23-editor-redraw-coalescing-damage-queue-architecture-review.md` owns region/reason transport
  through redraw and event-loop coalescing.

## Acceptance handoff

The handoff requires 33/33 post-change fingerprints, focused and managed Rust tests, pointer/move/tab/
window/target/resize/backend/scale matrices, same-executable WPR artifacts on D/E/F, RenderDoc GPU/
scissor/pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
