---
title: Runtime398 Prepared Queue Group Capacity
category: zircon_runtime
report_id: Runtime398-prepared-queue-group-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime398 Prepared Queue Group Capacity

Prepared mesh queue statistics now consume the input iterator once, use its lower size hint, and
reserve candidate-group hash capacity before classifying draws. All phase, shadow, velocity,
skinning, indirect, LOD, and repeated-group counters retain their existing behavior while large
draw queues avoid repeated hash-table growth and rehashing.

The ignored Windows Release benchmark emits `RUNTIME398_PREPARED_QUEUE_GROUP_CAPACITY_BENCH_V1`
over 17 alternating paired samples with 65,536 draw keys and 32,768 unique groups, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime398 is submitted with Runtime399 under request
`runtime398-runtime399-performance-batch-20260830cw-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
