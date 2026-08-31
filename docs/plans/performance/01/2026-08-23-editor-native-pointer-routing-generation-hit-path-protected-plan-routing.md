---
title: Editor native pointer routing generation and hit-path protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-pointer-routing-generation-hit-path-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the obsolete 2026-07-17 routing portion with one concise `pending.md` entry:

`zircon_editor retained_host/native_pointer/routing (53/53 current Rust files): M0 split-generation/
borrowed-row/topmost fixes; unified hit path, spatial indexes, typed ids, WPR/power and RenderDoc
parity pending.`

Move it to `review.md` only after M0-M4 pass on one source/executable fingerprint. Detailed findings
remain in the owner report, not the protected ledgers.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Supersede the stale asset-node linear-scan claim with retained asset paint metadata. Keep current
row cloning, split-generation drift, unified path, typed ids and scale acceptance active.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one generation-coherent `HostPointerHitPath`, topmost ordering and bubble/tunnel reply routing.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own convergence of the Workbench hit index with chrome/pane/runtime-surface hit ancestry.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own chrome/pane spatial indexes, Console scrolled geometry parity and bounded candidate visits.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own atomic structure/interaction/hit-index generation publication so routing never observes reset
or mixed-generation fields.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable paint-time hit grids, stable route ids and retained hit-path contracts.

## Acceptance handoff

The handoff requires 53/53 post-change fingerprints, focused and managed Rust tests, full event/
scale/overlap/Console matrices, same-executable WPR artifacts on D/E/F, RenderDoc parity, milestone
commit and quantified WeCom notification. Protected ledgers remain unchanged until then.
