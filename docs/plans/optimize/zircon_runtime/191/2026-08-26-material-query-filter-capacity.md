# Runtime191 Material Query Filter Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime191-editor137-performance-batch-20260826ev-v1`

## Problem

Material-management queries can emit at most status, issue-kind, and text filter rows, but the
short-lived output vector grew from zero capacity on fully filtered queries.

## Optimization

- Allocate once to the fixed maximum of three active filters.
- Preserve status/issue/text ordering, normalization, and each row's filter-removal query.

## Regression Contract

The `optimization_batch_20260826ev_` Runtime tests cover all three filter kinds and remove-query
semantics, source shape, and an ignored paired release benchmark emitting
`RUNTIME191_MATERIAL_QUERY_FILTER_CAPACITY_BENCH_V1`. It writes three real filter values 174,763
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
