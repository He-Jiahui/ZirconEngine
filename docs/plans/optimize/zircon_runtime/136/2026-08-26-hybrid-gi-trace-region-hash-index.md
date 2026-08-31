# Runtime136 Hybrid GI Trace Region Hash Index

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime136-editor82-performance-batch-20260826cs-v1`

## Problem

Hybrid-GI post-process encoding rebuilt a `BTreeMap` for prepared region lookup and a `BTreeSet`
for scheduled-ID deduplication every enabled frame. Neither container's order determined output;
the bounded schedule remained the sole encoding order.

## Optimization

- Build the per-frame region lookup as a `HashMap` from the exact-size scene-data iterator.
- Deduplicate scheduled IDs with a `HashSet` preallocated to the bounded schedule size.
- Preserve last-duplicate region indexing, first-scheduled-ID admission, missing-region skipping,
  and schedule-driven GPU array order.

## Regression Contract

The shared `optimization_batch_20260826cs_` filter owns three Runtime tests: duplicate-index
behavior, source shape, and an ignored paired release P95 benchmark. The benchmark emits
`RUNTIME136_HYBRID_GI_TRACE_REGION_HASH_INDEX_BENCH_V1`, indexes 16,384 regions and 8,192 scheduled
IDs per sample, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
