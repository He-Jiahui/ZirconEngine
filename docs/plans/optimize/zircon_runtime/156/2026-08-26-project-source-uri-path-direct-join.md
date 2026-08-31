# Runtime156 Project Source URI Path Direct Join

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime156-editor102-performance-batch-20260826dm-v1`

## Problem

Project and package asset URI generation separately collected relative path components into a
temporary vector before joining the portable URI path. Import scans and package resolution repeat
these conversions across many asset records.

## Optimization

- Share both URI call sites through one borrowed `relative_uri_path` helper.
- Reserve from the source encoded path length and append components with `/` separators directly.
- Preserve component normalization, lossy platform conversion, empty paths, and ordering.

## Regression Contract

The shared `optimization_batch_20260826dm_` filter owns three Runtime tests: component behavior,
shared direct-join source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME156_PROJECT_SOURCE_URI_PATH_DIRECT_JOIN_BENCH_V1`, renders 16,384 relative paths with
32 components per sample, removes one temporary component vector per path, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
