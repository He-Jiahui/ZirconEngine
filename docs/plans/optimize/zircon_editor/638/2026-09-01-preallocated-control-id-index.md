---
title: Editor638 Preallocated Control ID Index
category: zircon_editor
report_id: Editor638-preallocated-control-id-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor638 Preallocated Control ID Index

Preview control-ID indexing now reserves its borrowed HashMap from the lower-bound size hint of the
document node iterator. First duplicate control IDs remain admitted through `or_insert`, preserving
the prior projection's first-node semantics while avoiding repeated table growth for large previews.

The ignored Windows Release benchmark emits `EDITOR638_PREALLOCATED_CONTROL_ID_INDEX_BENCH_V1`
over 17 alternating sample pairs with 65,536 nodes. The gate requires preallocated P95 to be at most
80% of the unreserved index-build P95.

No direct Cargo validation was run. The coordinator owns combined Runtime638/Editor638 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor638 is prepared with Runtime638 under the shared `optimization_batch_iz_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
