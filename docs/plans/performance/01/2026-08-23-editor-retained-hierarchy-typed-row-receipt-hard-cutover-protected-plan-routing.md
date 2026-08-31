---
title: Editor retained hierarchy typed row receipt hard cutover protected routing
date: 2026-08-23
status: routing_requested_m0_static_validated
owner_record: 2026-08-23-editor-retained-hierarchy-typed-row-receipt-hard-cutover-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/hierarchy_pointer (22/22 reviewed; 15/15 current Rust files): current
O(1) row hit and O(V) paint verified; M0 typed-row Arc/mirror-hit/String hard cut statically
validated; generation receipt, managed Rust tests, WPR/allocator/power pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own all-row pointer projection allocation counts, mirror dispatch removal, scale/storm/WPR/power
matrices and final quantitative evidence.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own the single native hierarchy pane hit receipt and future presentation-generation field.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own retained typed hierarchy row identity shared by native paint, input and command dispatch.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own deletion of the generic hierarchy mirror hit surface and direct O(1) row routing contract.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own hierarchy topology/filter publication generation and interaction-only invalidation behavior.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed retained list-item identity and view-bounded generation; do not restore string identity
mirrors or logical-row hit nodes.

## Acceptance handoff

Require post-cutover fingerprints, focused and managed Rust tests, D/E/F WPR/allocator/power
artifacts, selection/rename/drag equivalence, RenderDoc pixel/draw parity, milestone commit and
quantified WeCom notification. Protected ledgers remain unchanged until then.

Current static evidence: owner files `22 -> 15`, lines `628 -> 360`, bytes `21,643 -> 11,863`;
pointer-projection row clones and identifier allocations `N -> 0`; generic mirror dispatches
`1 -> 0`; route String clones/NodeId parses `1 -> 0`. Focused RED 0/9 to GREEN 9/9,
retained-host contracts 49/49, profile-capture Pester 45/45, Rustfmt and scoped diff check passed.
Broad current-worktree discovery is 220/222 because a separate active asset-preview test references
an unfinished missing source/helper. Managed Cargo is blocked by archived Session
`validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552`. No executable, timing, power, WPR or
RenderDoc acceptance is claimed.
