---
title: Editor retained welcome recent typed item receipt hard cutover protected routing
date: 2026-08-23
status: routing_requested_m0_static_validated
owner_record: 2026-08-23-editor-retained-welcome-recent-typed-item-receipt-hard-cutover-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/welcome_recent_pointer (21/21 reviewed; 15/15 current Rust files):
current O(1) row hit and O(V) paint verified; M0 direct receipt/path clone/state publication cut
statically validated; typed item generation, managed Rust tests, WPR/allocator/power pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own click-time snapshot/path-clone counts, mirror dispatch removal, scale/storm/WPR/power matrices
and final quantitative evidence.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own the single native Welcome pane hit receipt, Copy action route and future presentation generation.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own one typed recent-project item allocation shared by Welcome presentation, paint and input.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own deletion of the generic Welcome mirror hit surface and direct O(1) routing contract.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own recent-project generation publication and changed-only hover/scroll invalidation.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed retained list-item identity and view-bounded generation; do not restore path identity
mirrors or logical-row hit nodes.

## Acceptance handoff

Require post-cutover fingerprints, focused and managed Rust tests, D/E/F WPR/allocator/power
artifacts, open/remove/hover/scroll equivalence, RenderDoc pixel/draw parity, milestone commit and
quantified WeCom notification. Protected ledgers remain unchanged until then.

Current static evidence: owner files `21 -> 15`, lines `689 -> 417`, bytes `26,195 -> 14,951`;
mirror dispatch `1 -> 0`, click snapshot/path projection `1 + N -> 0`, route path clones up to
`2 -> 0`, state owners `2 -> 1`, stable UI setters up to `6 -> 0`. Focused RED 1/11 to GREEN
11/11, retained-host contracts 70/70, broad performance contracts 248/248, profile Pester 45/45,
Rustfmt and scoped diff check passed. Managed Cargo is blocked by archived Session
`validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552`. No executable, timing, power, WPR or
RenderDoc acceptance is claimed.
