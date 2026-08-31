# Editor104 Hybrid GI Profile Borrowed Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime158-editor104-performance-batch-20260826do-v1`

## Problem

Editor viewport Hybrid GI profile parsing allocated a lowercase copy of the environment token
before matching a fixed set of ASCII profile names. The process cache bounds repeated environment
reads, but the parser itself did not need ownership.

## Optimization

- Trim once and compare each accepted spelling with `eq_ignore_ascii_case`.
- Preserve both hyphenated and underscored product-profile tokens.
- Preserve rejection of spaces and unsupported spellings without allocating.

## Regression Contract

The shared `optimization_batch_20260826do_` filter owns three Editor tests: token behavior,
borrowed source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR104_HYBRID_GI_PROFILE_BORROWED_PARSE_BENCH_V1`, performs 262,144 parses per sample, records
allocations per parse from one to zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
