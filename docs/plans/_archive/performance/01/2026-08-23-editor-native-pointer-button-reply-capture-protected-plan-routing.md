---
title: Editor native pointer button reply and capture protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-pointer-button-reply-capture-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 button-dispatch portion with one concise `pending.md` entry:

`zircon_editor retained_host/native_pointer/button_dispatch (105/105 current Rust files): M0
capture/no-change allocation fixes; typed reply, one-route dispatch, exact damage, WPR/power and
RenderDoc parity pending.`

Move it to `review.md` only after M0-M4 and dynamic acceptance pass on one source/executable
fingerprint. Do not expand either protected ledger with per-file details.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Supersede PERF-MVP-176's deep-snapshot premise with the current generation-handle finding. Retain
captured-release ordering, typed reply, duplicate callback traversal, exact damage and dynamic
acceptance as the active work.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one retained route and a typed reply carrying handled/capture/drag/focus/frame-update/damage
effects. Require captured release before ordinary hit-path construction, following Unreal's
captor-first pointer-up path.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own Workbench/template activation replies and remove unconditional release-row damage once pressed
state and exact invalidation receipts exist.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own one-generation hit-path construction, typed route identities and one hit build per uncaptured
button fact.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own the one-publication/one-frame-update effect contract and prevent bool/void callbacks from
forcing conservative redraw.

## `docs/plans/zircon_editor/editor/09-editor-asset-management.md`

Own the merged asset press reply so drag detection and selection reuse one prepared target and one
bridge traversal.

## `docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`

Own the merged hierarchy press reply so drag-source detection, selection, rename tracking and exact
damage share one retained route.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable pointer replies, retained hit paths and multi-region invalidation semantics shared by
runtime and editor UI.

## Acceptance handoff

The handoff requires 105/105 post-change fingerprints, focused and managed Rust tests, full
button/state/capture/target/row/backend/scale matrices, same-executable WPR artifacts on D/E/F,
RenderDoc draw/scissor/pixel parity, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
