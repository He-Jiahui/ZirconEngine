# Runtime223 Render Profile Expansion Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime223-editor169-performance-batch-20260826gb-v1`

## Problem

Runtime render-profile validation rebuilt the owner/include expansion Vec from empty even though the
owner plus direct include count is known before nested implied profiles are deduplicated.

## Optimization

- Preallocate the required-profile Vec from `includes.len() + 1` using saturating arithmetic.
- Preserve owner-first order, direct include order, implied-profile expansion, and duplicate removal.
- Treat the capacity as a lower bound so nested implied profiles can still grow without changing
  validation or capability semantics.

## Regression Contract

The `optimization_batch_20260826gb_` Runtime tests cover default-render expansion order/capacity and
the production reserve contract, and provide an ignored paired release benchmark emitting
`RUNTIME223_RENDER_PROFILE_EXPANSION_CAPACITY_BENCH_V1`. It builds 8,192 five-profile expansions per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
