# Runtime222 Shader Module Projection Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime222-editor168-performance-batch-20260826ga-v1`

## Problem

Native plugin shader-module loading grew both its accepted-source Vec and import-path HashSet from
empty even though the capped declaration count is known before module validation begins.

## Optimization

- After package-root validation, compute `min(declared modules, 64)` once and preallocate both hot
  collections from that bound.
- Keep early error paths allocation-free for accepted sources and leave diagnostics demand-grown.
- Preserve declaration order, duplicate detection, path confinement, size budgets, UTF-8 checks,
  source identities, and the 64-module processing limit; invalid modules may leave spare capacity.

## Regression Contract

The `optimization_batch_20260826ga_` Runtime tests cover capped capacity and the two production
reserves, and provide an ignored paired release benchmark emitting
`RUNTIME222_SHADER_MODULE_PROJECTION_CAPACITY_BENCH_V1`. It builds 4,096 projections of 64 modules
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
