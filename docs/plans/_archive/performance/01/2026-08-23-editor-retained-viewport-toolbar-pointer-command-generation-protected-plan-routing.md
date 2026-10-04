---
title: Editor retained viewport toolbar pointer command generation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-retained-viewport-toolbar-pointer-command-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace obsolete viewport-toolbar pointer text with one concise `pending.md` entry:

`zircon_editor retained_host/viewport_toolbar_pointer (31/31 pre-M0 Rust files): frame identity,
single hit/command authority, incremental topology, WPR/power and pixel parity pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint. Detailed findings
remain in the owner report, not the protected ledgers.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own same/new-frame scan counts, classifier/allocation counts, chrome snapshot counts, surface rebuild
counts, click storms, WPR/power and RenderDoc parity.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Close or supersede `failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md` only after the
producer cache, host presentation and pointer consumer share one generation-owned surface handle.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own stable native hit/command receipts and removal of the second pointer hit test.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own one authoritative toolbar hit path, receipt freshness and stale-generation rejection.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own per-surface generation publication and changed-surface callbacks without full presentation scan.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own atomic node/handler/route topology updates and incremental hit-index integration.

## `docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`

Own typed compact command identities analogous to Unreal's persistent viewport command list.

## `docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`

Reconcile the open `failure-2026-08-23-editor01-viewport-toolbar-cache-signature-move.md`: current
source updates the cached signature in place, but managed current-source validation is still blocked.

## Acceptance handoff

The handoff requires post-change owner fingerprints, focused and managed Rust tests, scale/storm WPR
artifacts on D/E/F, behavior/pixel parity, milestone commit and quantified WeCom notification.
Protected ledgers remain unchanged until then.
