# Runtime144 Mobility Borrowed Enum Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime144-editor90-performance-batch-20260826da-v1`

## Problem

Every reflected Mobility property write allocated an ASCII-lowercase copy of its owned enum value
before matching `dynamic` or `static`. The reflection writer already owns the original value for
error reporting, so this temporary allocation only served keyword dispatch.

## Optimization

- Extract a borrowed Mobility parser using trimmed `eq_ignore_ascii_case` comparisons.
- Reuse that parser from the reflection writer without cloning or normalizing the owned input.
- Preserve mixed-case/whitespace acceptance, unchanged-value reporting, successful updates, type
  mismatch errors, and unsupported-conversion errors containing the original value.

## Regression Contract

The shared `optimization_batch_20260826da_` filter owns three Runtime tests: enum behavior,
zero-owned-lowercase source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME144_MOBILITY_BORROWED_ENUM_PARSE_BENCH_V1`, executes 262,144 parses per sample,
records keyword allocations from 262,144 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
