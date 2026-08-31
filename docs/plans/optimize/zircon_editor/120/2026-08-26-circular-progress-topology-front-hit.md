# Editor120 Circular Progress Topology Front Hit

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime174-editor120-performance-batch-20260826ee-v1`

## Problem

The retained circular-progress renderer uses a four-entry LRU topology cache. Even when consecutive
frames requested the already-front topology, lookup still searched the deque, removed index zero,
and pushed the same `Rc` back to the front.

## Optimization

- Check the front entry before the LRU search and return an `Rc` clone without deque mutation.
- Search from the second entry only for non-front hits.
- Preserve move-to-front ordering, miss construction, and the four-entry cache bound.

## Regression Contract

The shared `optimization_batch_20260826ee_` filter owns three Editor tests: front/LRU ordering,
front-before-search source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `EDITOR120_CIRCULAR_PROGRESS_TOPOLOGY_FRONT_HIT_BENCH_V1`, performs 524,288 front hits per
sample, reduces deque mutations per front hit from two to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
