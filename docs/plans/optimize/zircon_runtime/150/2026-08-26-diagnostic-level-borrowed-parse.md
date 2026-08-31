# Runtime150 Diagnostic Level Borrowed Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime150-editor96-performance-batch-20260826dg-v1`

## Problem

Diagnostic log-level parsing lowercased every complete input into a temporary `String` before
matching a fixed alias set. Runtime configuration and filter parsing therefore allocated even for
already canonical values.

## Optimization

- Compare the trimmed borrowed input with fixed aliases using `eq_ignore_ascii_case`.
- Preserve verbose/trace, log/info, warn/warning, error/err, and off/none/quiet aliases.
- Preserve unknown-value diagnostics with the original trimmed spelling.

## Regression Contract

The shared `optimization_batch_20260826dg_` filter owns three Runtime tests: alias/error behavior,
allocation-free source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME150_DIAGNOSTIC_LEVEL_BORROWED_PARSE_BENCH_V1`, parses 262,144 aliases per sample, records
lowercase allocations from 262,144 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
