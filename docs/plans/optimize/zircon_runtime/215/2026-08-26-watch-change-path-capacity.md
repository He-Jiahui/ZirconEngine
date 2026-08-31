# Runtime215 Watch Change Path Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime215-editor161-performance-batch-20260826ft-v1`

## Problem

Incremental asset-registry rebuilds appended every non-removed watcher URI into a vector grown from
empty even though the watcher batch length was an immediate lower bound for the common no-remint
path.

## Optimization

- Reserve the watcher change count before projecting changed asset paths; duplicate-GUID remints
  can still grow the vector when they add paths beyond that lower bound.
- Preserve removed-change filtering, URI order, reminted-path de-duplication, replacement behavior,
  diagnostics, persistence, and atomic-fault handling.

## Regression Contract

The `optimization_batch_20260826ft_` Runtime tests cover capacity and URI order, enforce the
production reservation site, and provide an ignored paired release benchmark emitting
`RUNTIME215_WATCH_CHANGE_PATH_CAPACITY_BENCH_V1`. It projects 64 batches of 4,096 changes per sample
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
