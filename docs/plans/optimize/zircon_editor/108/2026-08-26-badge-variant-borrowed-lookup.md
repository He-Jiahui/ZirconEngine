# Editor108 Badge Variant Borrowed Lookup

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime162-editor108-performance-batch-20260826ds-v1`

## Problem

Retained badge projection cloned `variant` or `mui_variant` into a new string only to compare it
with `dot`. Every badge projection with a string variant allocated even though the attribute map
already owned stable text for the duration of the check.

## Optimization

- Borrow string variants directly from the TOML attribute value.
- Use the static `standard` fallback without allocation.
- Preserve dot badge suppression, non-string fallback, and non-badge behavior.

## Regression Contract

The shared `optimization_batch_20260826ds_` filter owns three Editor tests: projection behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR108_BADGE_VARIANT_BORROWED_LOOKUP_BENCH_V1`, performs 524,288 lookups per sample, reduces
allocations per check from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
