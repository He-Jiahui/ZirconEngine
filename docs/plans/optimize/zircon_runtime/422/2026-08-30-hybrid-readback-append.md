---
title: Runtime422 Hybrid Readback Append
category: zircon_runtime
report_id: Runtime422-hybrid-readback-append-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime422 Hybrid Readback Append

Hybrid-GI renderer and sideband readback merges now use `Vec::append` for every owned vector. The
sideband vectors are moved into the merge function, so appending transfers their allocations and
preserves the existing renderer-first order without per-element moves. Scalar counters, optional
Global SDF authority, and scene-prepare fallback behavior remain unchanged.

The previous implementation used `extend` on each owned sideband vector, iterating and moving every
element into the renderer vector. The new path transfers each source allocation directly; empty
and one-sided readbacks still return through the existing early exits.

The ignored Windows Release benchmark emits
`RUNTIME422_HYBRID_READBACK_APPEND_BENCH_V1` over 17 alternating paired samples, each merging
8,192 vectors of 4,096 values, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime422 is prepared with Editor350 under request
`runtime422-editor350-performance-batch-20260830dn-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
