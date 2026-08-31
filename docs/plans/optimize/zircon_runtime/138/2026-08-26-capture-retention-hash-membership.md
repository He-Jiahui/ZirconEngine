# Runtime138 Capture Retention Hash Membership

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime138-editor84-performance-batch-20260826cu-v1`

## Problem

Every retained capture preview built a `BTreeSet<&str>` from the retained slot identifiers before
projecting the virtual archive manifest. The set was used only for membership tests; canonical
manifest order already came from the explicit final `slot_id` sort.

## Optimization

- Replace the ordered tree with a capacity-reserved borrowed `HashSet<&str>`.
- Keep slot identifiers borrowed from the prune report, so the index creates no identifier copies.
- Preserve self-slot replacement, retained-slot filtering, manifest metadata, and the explicit
  canonical `slot_id` output sort.

## Regression Contract

The shared `optimization_batch_20260826cu_` filter owns three Runtime tests: membership and
deduplication, canonical projection order, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME138_CAPTURE_RETENTION_HASH_MEMBERSHIP_BENCH_V1`, builds 8,192 retained
identifiers, executes 12 dense probe rounds, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
