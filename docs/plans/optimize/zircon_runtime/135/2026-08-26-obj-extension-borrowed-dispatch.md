# Runtime135 OBJ Extension Borrowed Dispatch

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime135-editor81-performance-batch-20260826cr-v1`

## Problem

Every path-backed mesh load lowercased its extension into a new `String` before dispatch. The only
supported format is OBJ, so the successful path paid for an owner that was immediately discarded.

## Optimization

- Dispatch OBJ extensions with borrowed `eq_ignore_ascii_case` matching.
- Allocate a normalized extension only on the unsupported-format diagnostic path, where the typed
  error must own it.
- Preserve case-insensitive OBJ support and the existing lowercase diagnostic payload.

## Regression Contract

The shared `optimization_batch_20260826cr_` filter owns three Runtime tests: extension behavior,
source shape, and an ignored paired release P95 benchmark. The benchmark emits
`RUNTIME135_OBJ_EXTENSION_BORROWED_DISPATCH_BENCH_V1`, performs 240,000 supported extension checks
per sample, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
