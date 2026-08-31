# Runtime247 Unique Entity Hash Scan

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime247-editor193-performance-batch-20260826ha-v1`

## Problem

Large ECS entity arrays copied every entity together with its request index, sorted the tuple array,
scanned adjacent pairs, and then searched all duplicate pairs for the earliest request position. The
index-preserving sort paid `O(n log n)` work even when all requested entities were unique.

## Optimization

- Preserve the allocation-free pairwise scan for arrays of at most 16 entities.
- For larger arrays, preallocate a hash set and scan entity ids once in request order.
- Return immediately when insertion finds the first repeated id, preserving duplicate diagnostics.

## Regression Contract

The `optimization_batch_20260826ha_` Runtime tests preserve the first duplicate in request order,
enforce the large-array hash-scan dispatch, and provide an ignored paired release benchmark emitting
`RUNTIME247_UNIQUE_ENTITY_HASH_SCAN_BENCH_V1`. It repeatedly validates 8,192 unique entity ids and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70` against the former tuple-sort implementation.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
