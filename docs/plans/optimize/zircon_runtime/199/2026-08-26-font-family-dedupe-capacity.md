# Runtime199 Font Family Dedupe Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime199-editor145-performance-batch-20260826fd-v1`

## Problem

Runtime font matching grew both the family identity set and the ordered result vector from empty
while consuming an iterator whose lower size bound was already available.

## Optimization

- Convert the family source into its iterator once and reserve its lower size bound in both output
  collections before preserving empty-family filtering, case-insensitive deduplication, and order.
- Keep the generic iterator contract intact; iterators without a useful lower bound retain the
  existing growth behavior.

## Regression Contract

The `optimization_batch_20260826fd_` Runtime tests cover empty-family filtering, normalized
deduplication, stable order, result capacity, source shape, and an ignored paired release benchmark
emitting `RUNTIME199_FONT_FAMILY_DEDUPE_CAPACITY_BENCH_V1`. It inserts 256 lightweight identities
2,048 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
