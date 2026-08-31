# Runtime202 Project Feature Provider Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime202-editor148-performance-batch-20260826fg-v1`

## Problem

Runtime plugin catalog matching grew the project feature-provider lookup from an empty hash map
while every selection's feature row count was already available.

## Optimization

- Reserve the saturating total feature row count across project plugin selections before inserting
  borrowed feature and provider identities.
- Preserve first-provider-wins behavior for duplicate feature ids and avoid new owned strings.

## Regression Contract

The `optimization_batch_20260826fg_` Runtime tests cover four selections with 256 unique features,
a duplicate feature with conflicting providers, borrowed lookup results, source shape, and an
ignored paired release benchmark emitting `RUNTIME202_PROJECT_FEATURE_PROVIDER_CAPACITY_BENCH_V1`.
It inserts 256 lightweight map entries 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
