# Editor101 Autosave Source Path Direct Join

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime155-editor101-performance-batch-20260826dl-v1`

## Problem

Autosave source-path admission validated project-relative components, then collected borrowed UTF-8
component strings into a temporary vector before joining the persisted portable path. Snapshot
catalog updates repeated the extra allocation for every accepted source.

## Optimization

- Keep the existing relative/UTF-8/normal-component validation gate unchanged.
- Reserve one result buffer from the encoded path length and append components with `/` separators.
- Share normalization through a borrowed helper that does not retain a component collection.

## Regression Contract

The shared `optimization_batch_20260826dl_` filter owns three Editor tests: validation behavior,
direct-append source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR101_AUTOSAVE_SOURCE_PATH_DIRECT_JOIN_BENCH_V1`, normalizes 16,384 paths with 32 components
per sample, removes one temporary component vector per path, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
