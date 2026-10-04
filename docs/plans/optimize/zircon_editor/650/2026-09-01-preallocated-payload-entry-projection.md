---
title: Editor650 Preallocated Payload Entry Projection
category: zircon_editor
report_id: Editor650-preallocated-payload-entry-projection-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor650 Preallocated Payload Entry Projection

Binding payload projection now reserves its flattened entry vector from the top-level payload map
size. Every top-level value contributes at least one projected entry, making this a conservative
lower bound; nested arrays and maps continue appending through the existing recursive traversal in
the same BTreeMap and child order.

The ignored Windows Release benchmark emits `EDITOR650_PAYLOAD_ENTRY_CAPACITY_BENCH_V1` over 21
alternating sample pairs, 64 batches per sample, and 4,096 flat payload entries per batch. The gate
requires reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime650/Editor650 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor650 is prepared with Runtime650 under the shared `optimization_batch_jk_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
