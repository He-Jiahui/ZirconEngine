# Editor89 Promotion Document ID In-Place Trim

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime143-editor89-performance-batch-20260826cz-v1`

## Problem

UI asset component promotion built a normalized document ID, then called
`trim_matches('.').to_string()` to allocate and copy a second result. The normalizer already
collapses separators, so the only possible trailing delimiter is one dot.

## Optimization

- Reserve the normalized ID buffer from the trimmed input byte length.
- Remove the single possible trailing dot in place and return the original buffer.
- Preserve lowercase/digit/underscore acceptance, uppercase folding, separator collapse, leading
  separator rejection, non-ASCII handling, and empty-result rejection.
- Cover both create and update promotion paths through their shared normalizer.

## Regression Contract

The shared `optimization_batch_20260826cz_` filter owns three Editor tests: legacy-output parity,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR89_PROMOTION_DOCUMENT_ID_IN_PLACE_TRIM_BENCH_V1`, normalizes 8,192 representative document
IDs, records the per-document allocation reduction from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
