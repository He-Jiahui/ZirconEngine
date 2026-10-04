---
title: Editor retained detail scroll direct change receipt hard cutover protected routing
date: 2026-08-23
status: routing_requested_m0_static_validated
owner_record: 2026-08-23-editor-retained-detail-scroll-direct-change-receipt-hard-cutover-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/detail_pointer (23/23 reviewed; 19/19 current Rust files): direct
scalar clamp/change receipt and changed-only setters statically validated; managed Rust tests,
WPR/allocator/power pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own detail-scroll setter/rebuild/state-copy counts, input storm/WPR/power matrices and final
quantitative evidence.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own native pane-specific scroll receipt and explicit changed/unchanged propagation.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own one retained scroll state shared by host publication and paint for root/floating panes.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own deletion of detail mirror surfaces and direct viewport containment.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own changed-only scroll property publication and exact local invalidation/damage.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own scalar-scroll semantics; generic `UiSurface` dispatch is reserved for real child routing and
must not be restored for a single retained offset.

## Acceptance handoff

Require post-cutover fingerprints, focused and managed Rust tests, D/E/F WPR/allocator/power
artifacts, clamp/tail/header/window parity, RenderDoc pixel/draw parity, milestone commit and
quantified WeCom notification. Protected ledgers remain unchanged until then.

Current static evidence: owner files `23 -> 19`, lines `418 -> 275`, bytes `15,242 -> 9,872`;
generic mirror dispatches `1 -> 0`, retained state clones `>=1 -> 0`, unchanged property writes
`1 -> 0`. Focused RED 0/9 to GREEN 9/9 plus stable-sync RED 9/10 to GREEN 10/10; retained-host
contracts 59/59, Rustfmt and scoped diff check passed. Broad current-worktree discovery is 234/235
because a separate active asset-preview test rejects its eager shell projection. Managed Cargo is
blocked by archived Session `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552`. No executable,
timing, power, WPR or RenderDoc acceptance is claimed.
