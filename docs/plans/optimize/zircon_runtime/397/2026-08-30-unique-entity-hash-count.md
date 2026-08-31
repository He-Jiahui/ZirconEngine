---
title: Runtime397 Unique Entity Hash Count
category: zircon_runtime
report_id: Runtime397-unique-entity-hash-count-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime397 Unique Entity Hash Count

Virtual-geometry debug snapshot construction now counts unique visible entities with a
capacity-sized hash set instead of allocating and ordering a tree set. Instance authority over the
cluster fallback, duplicate handling, empty input behavior, and saturated `u32` reporting remain
unchanged. The count-only path no longer pays ordered-tree insertion and node-allocation costs.

The ignored Windows Release benchmark emits `RUNTIME397_UNIQUE_ENTITY_HASH_COUNT_BENCH_V1` over 17
alternating paired samples with 65,536 entity observations and 32,768 unique ids, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime397 is submitted with Runtime396 under request
`runtime396-runtime397-performance-batch-20260830cu-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
