# Runtime163 MUI Collection Borrowed Default Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime163-editor109-performance-batch-20260826dt-v1`

## Problem

MUI collection class generation cloned six defaulted string attributes before immediately writing
them into final class names. Image-list and table projections allocated for variant, position,
action position, size, and direction even though the template node retained the source text.

## Optimization

- Borrow and trim the first matching TOML string attribute from the node map.
- Return static defaults without allocation when the attribute is absent or empty.
- Apply the borrowed path to six defaulted collection class projections without changing the
  shared style compiler helper.

## Regression Contract

The shared `optimization_batch_20260826dt_` filter owns three Runtime tests: alias/trim behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME163_MUI_COLLECTION_BORROWED_DEFAULT_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per
sample, reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
