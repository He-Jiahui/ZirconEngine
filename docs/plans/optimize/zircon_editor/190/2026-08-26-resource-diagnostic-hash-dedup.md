# Editor190 Resource Diagnostic Hash Dedup

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime244-editor190-performance-batch-20260826gx-v1`

## Problem

UI asset resource resolution sorted every compiler and filesystem diagnostic before removing
duplicates by path, code, and message. Duplicate-heavy reports therefore paid full string sorting
cost for entries that were discarded immediately afterward.

## Optimization

- Preserve the existing stable sort and dedup path below 128 diagnostics.
- For larger reports, hash borrowed path, code, and message fields to mark the first retained entry
  without cloning diagnostic strings.
- Release the borrowed index, retain marked diagnostics in place, and sort only the unique result.

## Regression Contract

The `optimization_batch_20260826gx_` Editor tests preserve canonical identity ordering and the
existing severity-independent dedup contract, enforce hash-before-sort behavior, and provide an
ignored paired release benchmark emitting `EDITOR190_RESOURCE_DIAGNOSTIC_HASH_DEDUP_BENCH_V1`.
It repeatedly normalizes 2,048 diagnostics drawn from 16 unique identities and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
