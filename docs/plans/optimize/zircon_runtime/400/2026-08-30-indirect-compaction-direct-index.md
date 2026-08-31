---
title: Runtime400 Indirect Compaction Direct Index
category: zircon_runtime
report_id: Runtime400-indirect-compaction-direct-index-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime400 Indirect Compaction Direct Index

General indirect-compaction batch planning now allocates the final metadata array once and writes
each record directly at its source argument index. The coverage bitmap still rejects duplicate or
missing spans, while visible-instance prefixes and draw-count ownership remain assigned in batch
iteration order. Successful unordered plans no longer append in batch order and sort the complete
metadata array afterward.

The ignored Windows Release benchmark emits `RUNTIME400_INDIRECT_DIRECT_INDEX_BENCH_V1` over 17
alternating paired samples with 65,536 arguments and 2,048 reverse-ordered batches, requiring
`direct_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime400 is submitted with Runtime401 under request
`runtime400-runtime401-performance-batch-20260830cy-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
