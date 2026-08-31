---
title: Runtime402 GPU Dirty Range In-Place Sort
category: zircon_runtime
report_id: Runtime402-gpu-dirty-range-unstable-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime402 GPU Dirty Range In-Place Sort

GPU-scene dirty ranges now use an in-place unstable key sort before the existing linear merge. Equal
starts need no stable relative order because the merge keeps their shared start and maximum end, so
primitive, instance, and light upload byte ranges remain identical while large update batches avoid
the stable sorter scratch allocation.

The ignored Windows Release benchmark emits `RUNTIME402_GPU_DIRTY_RANGE_UNSTABLE_SORT_BENCH_V1`
over 17 alternating paired samples with 65,536 permuted dirty ranges, requiring
`unstable_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime402 is submitted with Runtime403 under request
`runtime402-runtime403-performance-batch-20260830cz-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
