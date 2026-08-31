# Runtime190 Material Selection ID Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime190-editor136-performance-batch-20260826eu-v1`

## Problem

Material selection deduplication grew both its ordered output vector and membership hash set while
iterating, even when the iterator exposed a guaranteed minimum item count.

## Optimization

- Convert once to an iterator and reserve both collections from its safe `size_hint` lower bound.
- Preserve first-occurrence ordering and duplicate elimination without trusting an optional upper bound.

## Regression Contract

The `optimization_batch_20260826eu_` Runtime tests cover 256 unique IDs plus duplicate requests,
selection hit/miss order, source shape, and an ignored paired release benchmark emitting
`RUNTIME190_MATERIAL_SELECTION_ID_CAPACITY_BENCH_V1`. It inserts 256 IDs 2,048 times per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
