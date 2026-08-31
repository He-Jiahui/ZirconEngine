# Runtime189 Navigation Triangle Mesh Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime189-editor135-performance-batch-20260826et-v1`

## Problem

Navigation triangle-mesh construction filtered input into index and polygon vectors that both grew
incrementally despite their maximum sizes being known from the input index count.

## Optimization

- Reserve the valid-index vector to the input index count.
- Reserve the polygon vector to `index_count / 3`, preserving invalid-triangle filtering and order.

## Regression Contract

The `optimization_batch_20260826et_` Runtime tests cover 256 valid triangles, output ordering and
capacity, source shape, and an ignored paired release benchmark emitting
`RUNTIME189_NAVIGATION_TRIANGLE_MESH_CAPACITY_BENCH_V1`. It writes 768 indices and 256 polygons 512
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
