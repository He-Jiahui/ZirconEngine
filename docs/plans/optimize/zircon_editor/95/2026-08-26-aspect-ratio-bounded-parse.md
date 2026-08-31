# Editor95 Aspect Ratio Bounded Parse

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime149-editor95-performance-batch-20260826df-v1`

## Problem

CSS-like auto-layout aspect-ratio parsing collected slash-delimited borrowed slices into a heap
`Vec`, even though the grammar accepts exactly one scalar or one numerator/denominator pair.
Every declaration update therefore allocated before numeric validation.

## Optimization

- Read at most two trimmed segments directly from the split iterator.
- Reject a third segment without materializing the complete sequence.
- Preserve scalar, ratio, empty-segment, non-negative, finite, and zero-denominator behavior.

## Regression Contract

The shared `optimization_batch_20260826df_` filter owns three Editor tests: bounded grammar,
allocation-free source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR95_ASPECT_RATIO_BOUNDED_PARSE_BENCH_V1`, parses 262,144 declarations per sample, records
temporary-vector allocations from 262,144 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
