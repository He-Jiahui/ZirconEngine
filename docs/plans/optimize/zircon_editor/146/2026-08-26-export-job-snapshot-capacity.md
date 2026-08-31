# Editor146 Export Job Snapshot Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime200-editor146-performance-batch-20260826fe-v1`

## Problem

The Editor export job queue grew its snapshot vector from empty even though pending job count and
the optional active-job slot exactly determine the final upper bound.

## Optimization

- Reserve `pending_count + active(0/1)` with saturating arithmetic before projecting snapshots.
- Preserve active-first ordering, cancellation phase selection, pending order, progress cloning,
  and profile-busy behavior.

## Regression Contract

The `optimization_batch_20260826fe_` Editor tests cover 256 queued snapshots, id and phase order,
empty/active/saturated capacity math, source shape, and an ignored paired release benchmark
emitting `EDITOR146_EXPORT_JOB_SNAPSHOT_CAPACITY_BENCH_V1`. It appends one active-shaped plus 255
pending-shaped lightweight entries 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
