---
title: Runtime459 Direct HTTP Byte Range
category: zircon_runtime
report_id: Runtime459-direct-http-byte-range-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime459 Direct HTTP Byte Range

HTTP range request construction now writes the fixed `bytes=start-end` grammar directly into one
preallocated string. Both `u64` bounds use stack-resident decimal buffers, removing generic
formatting from download chunk and retry descriptor construction.

Header replacement remains case-insensitive, unrelated headers retain their order, and decimal
output is unchanged for zero, ordinary chunk bounds, and the complete `u64` range. Regression
coverage exercises those boundaries and the full request mutation contract.

The ignored Windows Release benchmark emits `RUNTIME459_DIRECT_HTTP_BYTE_RANGE_BENCH_V1` over 17
alternating paired samples, each constructing 262,144 mixed range values. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime459 is prepared with Editor389 under request
`runtime459-editor389-performance-batch-20260831fa-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
