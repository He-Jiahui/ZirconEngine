# Runtime235 Reflection Snapshot Node Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime235-editor181-performance-batch-20260826go-v1`

## Problem

UI reflection snapshots appended one large reflected record per surface-tree node into an empty Vec,
causing repeated reallocations and moves while inspector and diagnostics snapshots were built.

## Optimization

- Preallocate the result Vec from the surface tree's exact node count.
- Preserve the original node iteration order, reflection projection, and hit-context construction.
- Keep the capacity calculation local to the reflection snapshot owner.

## Regression Contract

The `optimization_batch_20260826go_` Runtime tests cover the node-count capacity boundary and source
contract, and provide an ignored paired release benchmark emitting
`RUNTIME235_REFLECTION_SNAPSHOT_NODE_CAPACITY_BENCH_V1`. It builds 128 snapshots of 4,096
sixteen-field node payloads per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
