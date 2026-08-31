---
title: Runtime383 Primitive Picking Capacity
category: zircon_runtime
report_id: Runtime383-primitive-picking-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime383 Primitive Picking Capacity

Primitive picking now reserves the outer result from the ray count and each per-ray hit buffer from
the primitive count before intersection collection. Ray and primitive traversal order, hit
construction, backend ordering, and omission of empty hit groups remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME383_PRIMITIVE_PICKING_CAPACITY_BENCH_V1` over
17 paired samples with 64 rays and 256 primitives per ray, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime383 is submitted with Editor329 under request
`runtime383-editor329-performance-batch-20260830ce-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
