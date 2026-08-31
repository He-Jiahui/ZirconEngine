# Runtime313 Preallocated Static Shadow Casters

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime313-editor258-performance-batch-20260829am-v1`

## Problem

The resource-backed static shadow revision path appended every accepted mesh to an empty vector even
though the input mesh count was already the strict upper bound. Dense static scenes repeatedly grew
and copied this frame-local caster list before sorting and hashing it.

## Optimization

- Reserve the input mesh upper bound when the first valid caster is accepted.
- Preserve zero allocation when every input mesh is filtered out.
- Preserve enabled, shadow-casting, and static-mobility filtering.
- Preserve missing-resource failure, caster order normalization, and revision hashing.

## Regression Contract

The `optimization_batch_20260829am_` Runtime tests preserve list order and the empty-list fast path,
and guard the production reservation before the first append. The ignored paired release benchmark emits
`RUNTIME313_PREALLOCATED_STATIC_SHADOW_CASTERS_BENCH_V1`. It builds 10,000 512-caster lists per
sample, changes eight vector allocation operations to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
