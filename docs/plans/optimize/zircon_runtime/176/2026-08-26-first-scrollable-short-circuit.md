# Runtime176 First Scrollable Short Circuit

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime176-editor122-performance-batch-20260826eg-v1`

## Problem

The Runtime tree API resolved the first scrollable pointer-route candidate by collecting every
eligible candidate into a new vector and then taking its first element. A first-position hit still
visited the complete route, wrote every match, and allocated an output buffer.

## Optimization

- Scan candidates directly and return on the first eligible node.
- Reuse one scrollable-candidate predicate for first-match and collect-all APIs.
- Preserve ordering, visibility/enabled filtering, and missing-node errors before the first match.

## Regression Contract

The shared `optimization_batch_20260826eg_` filter owns three Runtime tests: ordered/error behavior,
non-collecting source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME176_FIRST_SCROLLABLE_SHORT_CIRCUIT_BENCH_V1`, performs 8,192 first-hit lookups per sample
over 256 candidates, reduces first-hit visits from 256 to one and output allocations from one to
zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
