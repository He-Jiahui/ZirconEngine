# Runtime210 BVH U32 List Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime210-editor156-performance-batch-20260826fo-v1`

## Problem

Virtual-geometry BVH graph dumps formatted every node's cluster-id list into a string grown from
the initial `[` allocation even though the element count and maximum decimal width of `u32` were
known before formatting.

## Optimization

- Reserve brackets, separators, and ten decimal bytes per value with saturating arithmetic before
  formatting the list.
- Preserve empty lists, numeric spelling, item order, separators, and graph dump output.

## Regression Contract

The `optimization_batch_20260826fo_` Runtime tests cover empty, mixed-width, and maximum `u32`
lists, verify the reserved upper bound and production source shape, and provide an ignored paired
release benchmark emitting `RUNTIME210_BVH_U32_LIST_CAPACITY_BENCH_V1`. It formats 1,024 lists of
64 maximum-width values per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
