# Runtime204 Package Feature Declaration Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime204-editor150-performance-batch-20260826fi-v1`

## Problem

Runtime plugin feature definition collection grew the declared feature-key set from empty while
all optional-feature and feature-extension counts were available across registration reports.

## Optimization

- Reserve the saturating total of both package feature lists across all registrations before
  inserting feature definition keys.
- Preserve package order, optional-before-extension order, duplicate definition diagnostics, and
  declared-key deduplication.

## Regression Contract

The `optimization_batch_20260826fi_` Runtime tests cover two packages with 64 optional features and
64 feature extensions each, definition order, declared-key cardinality, final capacity, source
shape, and an ignored paired release benchmark emitting
`RUNTIME204_PACKAGE_FEATURE_DECLARATION_CAPACITY_BENCH_V1`. It inserts 256 lightweight keys 2,048
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
