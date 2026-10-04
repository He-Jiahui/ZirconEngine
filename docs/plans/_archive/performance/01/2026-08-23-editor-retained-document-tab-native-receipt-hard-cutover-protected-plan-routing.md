---
title: Editor retained document tab native receipt hard cutover protected routing
date: 2026-08-23
status: routing_requested_m0_static_validated
owner_record: 2026-08-23-editor-retained-document-tab-native-receipt-hard-cutover-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Use one concise `pending.md` entry:

`zircon_editor retained_host/document_tab_pointer (19/19 reviewed; 12/12 current Rust files): M0
native receipt hard-cut statically validated; generation-owned identity, typed transaction,
managed Rust tests, WPR/power pending.`

Move it to `review.md` only after M0-M3 pass on one source/executable fingerprint.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Own mirror-hit/rebuild removal, receipt validation, projection allocation and scale/storm/WPR/power
matrices.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own the native document-tab action and generation receipt as the sole hit result.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own generation-owned document-tab identity shared by paint, input and command dispatch.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own removal of the editor mirror surface and stale-receipt behavior.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Own document topology generation publication and O(1) receipt validation.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own the single native hit authority contract; do not reintroduce a second tab hit tree.

## `docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`

Own compact typed document activation/close receipts and one command transaction.

## Acceptance handoff

Require post-cutover fingerprints, focused and managed Rust tests, D/E/F WPR artifacts,
interaction/pixel parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.

Current routing evidence: files `19 -> 12`, lines `740 -> 240`, bytes `29,998 -> 8,501`;
focused RED 1/6 to GREEN 6/6, retained-host contracts 23/23 and broad current-worktree performance
contracts 192/192. These are static gates only; no managed executable or profiler acceptance is
claimed.
