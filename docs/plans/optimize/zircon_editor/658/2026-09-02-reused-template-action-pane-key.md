---
title: Editor658 Reused Template Action Pane Key
category: zircon_editor
report_id: Editor658-reused-template-action-pane-key-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: admission_rejected_external_worktree_dirty
---

# Editor658 Reused Template Action Pane Key

Each template action slot now owns the same immutable pane/document/plugin key shape used by the
control-attribute index. Action lookup borrows that stored key instead of rebuilding a temporary
key and allocating pane/document strings for every invocation. Binding still materializes exactly
one key for the index and one equivalent key for the slot, so ownership, pane removal, document
removal, plugin-owner checks, and compiled-action behavior remain unchanged.

The deterministic source regression verifies that lookup borrows `slot.pane_key()` and does not
construct a key. The ignored Windows Release benchmark emits
`EDITOR658_REUSED_TEMPLATE_ACTION_PANE_KEY_BENCH_V1` over 17 alternating sample pairs and 500,000
lookups per sample. Its gate requires reused-key P95 to be at most 40% of allocated-key P95.

Aggregate coordinator admission was rejected before immutable ticket creation because an external
Git worktree is dirty. No Cargo validation or performance measurement ran. The coordinator must
validate this Editor candidate with retained Editor654 and Editor656 after admission is available;
no accepted optimization record, commit, push, or WeCom publication exists.
