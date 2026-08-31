# Runtime159 Migration Path Identity In-Place Lowercase

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime159-editor105-performance-batch-20260826dp-v1`

## Problem

Windows asset-migration recovery converted a resolved path to lossy text and then allocated another
string for ASCII lowercase identity. When lossy conversion already owned its buffer, every scanner
and recovery admission discarded that reusable allocation.

## Optimization

- Convert the lossy path `Cow` into one owned identity buffer.
- Apply `make_ascii_lowercase` in place and return the same allocation.
- Keep the non-Windows identity path unchanged and preserve non-ASCII bytes.

## Regression Contract

The shared `optimization_batch_20260826dp_` Windows filter owns three Runtime tests: identity
behavior, pointer/capacity reuse plus source shape, and an ignored paired release P50/P95 benchmark.
The benchmark emits `RUNTIME159_MIGRATION_PATH_IDENTITY_IN_PLACE_LOWERCASE_BENCH_V1`, normalizes
16,384 identities per sample, removes 16,384 lowercase allocations, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
