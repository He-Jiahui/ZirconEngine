---
title: Editor retained drawer header native receipt hard cutover protected routing
date: 2026-08-23
status: routing_requested_m0_static_validated
owner_record: 2026-08-23-editor-retained-drawer-header-native-receipt-hard-cutover-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/drawer_header_pointer (21/21 reviewed; 12/12 current Rust files): M0
native receipt hard-cut statically validated; generation-owned identity, typed transaction,
managed Rust tests, WPR/power pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own mirror-hit/rebuild removal, route clone counts, receipt validation and scale/storm/WPR/power
matrices.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own the native drawer-tab action and generation receipt as the sole hit result.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own generation-shared typed drawer slot/view identity across paint, input and command dispatch.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own removal of the drawer-header mirror surface and stale-receipt behavior.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own drawer topology publication generation and changed-owner invalidation.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the single native hit-authority contract; do not restore an editor drawer-header hit tree.

## Acceptance handoff

Require post-cutover fingerprints, focused and managed Rust tests, D/E/F WPR artifacts,
interaction/pixel parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.

Current routing evidence: files `21 -> 12`, lines `696 -> 224`, bytes `25,241 -> 7,678`;
focused RED 1/7 to GREEN 7/7, retained-host contracts 30/30 and broad current-worktree performance
contracts 199/199. Per-click template binding scan, slot parse and active drawer-map clone are
removed. These are static gates only; no managed executable or profiler acceptance is claimed.
