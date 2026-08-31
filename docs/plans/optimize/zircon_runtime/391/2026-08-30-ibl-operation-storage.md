---
title: Runtime391 IBL Operation Fixed Storage
category: zircon_runtime
report_id: Runtime391-ibl-operation-fixed-storage-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime391 IBL Operation Fixed Storage

Realtime IBL scheduling now stores its contractually single per-frame operation in a one-element
array instead of a heap `Vec`. The public slice view, operation ordering, idempotent same-frame
clone behavior, retry semantics, and generation publication state machine remain unchanged while
each newly materialized frame batch avoids one allocation.

The ignored Windows Release benchmark emits `RUNTIME391_IBL_OPERATION_STORAGE_BENCH_V1` over 17
alternating paired samples and 100,000 one-operation batches per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime391 is submitted with Editor335 under request
`runtime391-editor335-performance-batch-20260830cp-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
