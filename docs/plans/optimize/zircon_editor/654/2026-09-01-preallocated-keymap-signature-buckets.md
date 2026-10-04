---
title: Editor654 Preallocated Keymap Signature Buckets
category: zircon_editor
report_id: Editor654-preallocated-keymap-signature-buckets-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: admission_rejected_external_worktree_dirty
---

# Editor654 Preallocated Keymap Signature Buckets

Editor keymap construction now reserves the signature-index map from the effective binding count.
Every binding contributes one signature entry at most, while bucket ordering, collision behavior,
keyboard resolution, and override semantics remain unchanged.

The ignored Windows Release benchmark emits `EDITOR654_KEYMAP_SIGNATURE_INDEX_CAPACITY_BENCH_V1`
over 17 alternating sample pairs and 32 batches of 16,384 bindings. The gate requires reserved
P95 to be at most 80% of the unreserved growth path.

Runtime654 candidates were withdrawn after source review because one over-allocated common meshes
and the other duplicated iterator collection's allocation policy. Editor654 is retained as an
Editor-only candidate. No direct Cargo
validation, performance measurement, commit, push, or WeCom publication has succeeded; the
coordinator must validate this Editor slice after the current external-worktree admission blocker
is resolved.
