# Runtime246 Resource Diagnostic Hash Dedup

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime246-editor192-performance-batch-20260826gz-v1`

## Problem

Runtime UI resource dependency validation sorted every generated diagnostic before removing
duplicates by path, code, and message. Duplicate dependency declarations therefore paid full string
sorting cost for diagnostics that were immediately discarded.

## Optimization

- Preserve the existing stable sort and dedup path below 128 diagnostics.
- For larger reports, hash borrowed path, code, and message fields to mark the first retained entry
  without cloning diagnostic strings.
- Release the borrowed identity set, retain marked diagnostics in place, and sort only unique rows.

## Regression Contract

The `optimization_batch_20260826gz_` Runtime tests preserve canonical identity ordering and the
existing severity-independent dedup contract, enforce hash-before-sort behavior, and provide an
ignored paired release benchmark emitting `RUNTIME246_RESOURCE_DIAGNOSTIC_HASH_DEDUP_BENCH_V1`.
It repeatedly normalizes 2,048 diagnostics drawn from 16 identities and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
