---
title: Editor609 Linear Layer Key Merge
category: zircon_editor
report_id: Editor609-linear-layer-key-merge-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor609 Linear Layer Key Merge

Persistent settings-layer replacement now finds removed, added, and modified keys with a linear
two-cursor merge over the already sorted old and new `BTreeMap` entries. The previous implementation
chained both key iterators, cloned every changed key from both sides, and inserted those clones into
a `BTreeSet` to restore uniqueness and order.

The merge emits each changed key exactly once in global `SettingsKey` order. Identical reloads keep
an empty, unallocated result vector; validation still completes before mutation, layer replacement
remains atomic, and change-log revisions retain their prior deterministic ordering. Focused behavior
coverage mixes removals, additions, modifications, and unchanged keys. A source guard requires all
three ordered-merge branches and rejects the temporary ordered set.

The ignored Windows Release benchmark emits `EDITOR609_LINEAR_LAYER_KEY_MERGE_BENCH_V1` over 17
alternating sample pairs with 8,192 modified settings. Modeled changed-key clones fall from 16,384
to 8,192 and ordered-set insertions fall from 16,384 to zero. The gate requires linear-merge P95 to
be at most 20% of legacy P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor609 is prepared with Runtime609 under request
`runtime609-editor609-slot-layer-performance-20260901hz-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
