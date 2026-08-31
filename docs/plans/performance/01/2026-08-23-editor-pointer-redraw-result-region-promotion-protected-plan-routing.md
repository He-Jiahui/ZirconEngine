---
title: Editor pointer redraw-result region promotion protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-pointer-redraw-result-region-promotion-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the matching remainder of the obsolete 2026-07-17 native-pointer damage/redraw coverage with
one concise `pending.md` module entry:

`zircon_editor retained-host host_contract/native_pointer/{redraw_result.rs + redraw_result/**,resize_damage.rs,template_hover_damage.rs}`
- 14/14 current Rust files source-reviewed. Borrowed before/after generations, unchanged idle,
  hierarchy row arithmetic and frame-update distinction are retained. Pending M0-M4: typed mutation
  receipts and owner lookup, shared multi-region transport, old/new layout receipts, WPR/power/
  RenderDoc acceptance. No local Rust change because the current result contract cannot represent the
  required exact regions.

Do not add these files to `review.md` before M0-M4 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M4 to MVP pointer-to-present repaint cost. Record receipts/reasons/owners, state fields
compared, input/output regions, useful/union/submitted area, full/promotion reasons, CPU p50/p95/p99,
WPR scheduling/power and exact source/workload fingerprints.

## `docs/plans/performance/01/2026-08-23-editor-redraw-coalescing-damage-queue-architecture-review.md`

Own region/reason preservation through external redraw state, event-loop coalescing, retry and frame
request boundaries. Remove the one-frame `HostRedrawRequest::Region` authority after migration.

## `docs/plans/performance/01/2026-08-23-editor-frame-paint-geometry-damage-region-set-architecture-review.md`

Own the shared bounded `DamageRegionSet`, finite/visible normalization, overlap/promotion policy and
useful-versus-submitted area telemetry. Pointer result code must consume this type.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own typed pointer/hover/scroll transition receipts, old/current path identity and ordered capture/
leave semantics.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own stable route/owner ids and exact row/control hit-to-transition effects rather than full pane-frame
fallbacks.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own generation owner-to-frame/range projection and old/new layout transaction receipts for resize and
tab drop.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable reason-coded invalidation, retained owner/range propagation and multi-region redraw
contracts shared by runtime and editor UI.

## Acceptance handoff

The handoff requires 14/14 current fingerprints, focused and managed Rust tests, hover/row/region/
resize/drop/input/backend/scale matrices, same-executable WPR artifacts on D/E/F, RenderDoc GPU/
scissor/pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
