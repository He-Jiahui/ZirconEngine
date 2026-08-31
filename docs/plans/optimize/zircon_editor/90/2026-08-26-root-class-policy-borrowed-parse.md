# Editor90 Root Class Policy Borrowed Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime144-editor90-performance-batch-20260826da-v1`

## Problem

Each UI asset root-class policy edit allocated a lowercase string and then a second replacement
string before matching four fixed spellings. The normalized strings were discarded immediately
after the enum decision.

## Optimization

- Trim once and compare borrowed input with `eq_ignore_ascii_case`.
- Preserve `append_only`, `append-only`, `appendonly`, and `closed` spellings across mixed case.
- Leave editability checks, document cloning, replay construction, unchanged-value handling, and
  document application unchanged.

## Regression Contract

The shared `optimization_batch_20260826da_` filter owns three Editor tests: alias behavior,
zero-normalized-string source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `EDITOR90_ROOT_CLASS_POLICY_BORROWED_PARSE_BENCH_V1`, executes 262,144 parses per sample,
records per-parse allocations from two to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
