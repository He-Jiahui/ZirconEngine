# Runtime168 Autocomplete Borrowed Predicate Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime168-editor114-performance-batch-20260826dy-v1`

## Problem

MUI Autocomplete class generation cloned popup-icon and current-value attributes for node and
owner predicates. The four decision paths only matched text or checked its presence.

## Optimization

- Borrow and trim Autocomplete predicate attributes from node or owner maps.
- Preserve alias traversal, whitespace-only filtering, and free-solo/value fallbacks.
- Leave the owned tag-size path unchanged because it feeds final class formatting.

## Regression Contract

The shared `optimization_batch_20260826dy_` filter owns three Runtime tests: popup/value behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME168_AUTOCOMPLETE_BORROWED_PREDICATE_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per
sample, reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
