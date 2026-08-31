---
title: Runtime387 Picking Backend Capacity
category: zircon_runtime
report_id: Runtime387-picking-backend-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime387 Picking Backend Capacity

The per-frame picking pipeline now reserves backend output storage from the saturating product of
backend count and ray count, then extends the aggregate one backend result at a time. Backend call
order, each backend's output order, disabled ray-map behavior, hover resolution, and event
dispatch remain unchanged; a backend that emits beyond the estimate still grows the vector
normally.

The ignored Windows Release benchmark emits `RUNTIME387_PICKING_BACKEND_CAPACITY_BENCH_V1` over
17 paired samples with 32 backends and 128 outputs per backend, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime387 is submitted with Editor333 under request
`runtime387-editor333-performance-batch-20260830ck-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
