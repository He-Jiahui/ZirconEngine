---
title: Runtime409 Phase Queue Indirect Sort
category: zircon_runtime
report_id: Runtime409-phase-queue-indirect-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime409 Phase Queue Indirect Sort

Render phase queue construction now computes each ordering key once, unstable-sorts compact
`(ordering key, source index)` entries, and applies the resulting permutation to full queue items.
The source-index lane reproduces the previous stable ordering for equal keys while avoiding repeated
key construction and comparison-time movement of full `RenderPhaseItem` values.

The ignored Windows Release benchmark emits `RUNTIME409_PHASE_QUEUE_INDIRECT_SORT_BENCH_V1` over 17
alternating paired samples with 65,536 queue items, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime409 is submitted with Runtime408 under request
`runtime408-runtime409-performance-batch-20260830dc-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
