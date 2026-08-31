# Runtime171 MUI X Borrowed Owner Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime171-editor117-performance-batch-20260826eb-v1`

## Problem

MUI X slot generation cloned picker, chart, and chat owner strings before presence tests or final
class formatting. The local helper forced ownership across six call sites.

## Optimization

- Return trimmed borrowed owner strings from the local MUI X helper.
- Preserve alias traversal and whitespace-only filtering.
- Keep formatting ownership confined to the final class strings.

## Regression Contract

The shared `optimization_batch_20260826eb_` filter owns three Runtime tests: alias filtering,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME171_MUI_X_BORROWED_OWNER_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per sample, reduces
lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
