# Runtime141 Native Extension Borrowed Dispatch

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime141-editor87-performance-batch-20260826cx-v1`

## Problem

Native dynamic-library export discovery allocated an ASCII-lowercase `String` for every directory
entry extension before checking the six supported artifact kinds. Large native package directories
paid that allocation even though extension matching only needs ASCII case-insensitive equality.

## Optimization

- Dispatch by extension byte length before checking the small supported set.
- Compare borrowed extension text with `eq_ignore_ascii_case`.
- Preserve `dll`, `so`, `dylib`, `pdb`, `dbg`, and `dsym` support across mixed case while rejecting
  missing, compound, and unrelated extensions.

## Regression Contract

The shared `optimization_batch_20260826cx_` filter owns three Runtime tests: supported/rejected
extension behavior, zero-owned-lowercase source shape, and an ignored paired release P50/P95
benchmark. The benchmark emits `RUNTIME141_NATIVE_EXTENSION_BORROWED_DISPATCH_BENCH_V1`, executes
131,072 common Windows `.DLL` probes per sample, records extension allocations from 131,072 to zero,
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
