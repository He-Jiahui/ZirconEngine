# Editor138 Menu Route Index Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime192-editor138-performance-batch-20260826ew-v1`

## Problem

Menu popup route indexing grew both its route HashMap and reusable DFS path stack while traversing
a tree whose total node count and maximum depth are stable for the indexing operation.

## Optimization

- Compute total item count and maximum depth in one read-only DFS.
- Reserve the route map and path stack once while preserving depth-first route numbering.

## Regression Contract

The `optimization_batch_20260826ew_` Editor tests cover a 256-item two-level tree, route order and
shape, source shape, and an ignored paired release benchmark emitting
`EDITOR138_MENU_ROUTE_INDEX_CAPACITY_BENCH_V1`. It inserts 256 real route keys 2,048 times per sample
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
