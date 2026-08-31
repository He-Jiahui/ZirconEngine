# Runtime145 Metadata Tag In-Place Trim

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime145-editor91-performance-batch-20260826db-v1`

## Problem

Runtime session metadata normalization replaced every owned tag with `tag.trim().to_string()`.
That allocated and copied a second string for every tag even though the original owned buffer was
available for mutation.

## Optimization

- Truncate trailing whitespace and drain leading whitespace in the existing `String` allocation.
- Preserve Unicode whitespace trimming, empty-tag removal, lexical sorting, and deduplication.
- Keep allocation capacity stable while normalizing an owned tag.

## Regression Contract

The shared `optimization_batch_20260826db_` filter owns three Runtime tests: canonical output,
owned-buffer reuse, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME145_METADATA_TAG_IN_PLACE_TRIM_BENCH_V1`, normalizes 16,384 tags per sample, records trim
allocations from 16,384 to zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
