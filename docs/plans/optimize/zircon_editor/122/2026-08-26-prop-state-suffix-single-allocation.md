# Editor122 Prop State Suffix Single Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime176-editor122-performance-batch-20260826eg-v1`

## Problem

UI asset inspector projection formatted widget property/state kind and path into a complete
temporary string, then scanned that string to build a second sanitized control suffix. Every
projected row therefore allocated twice and decoded ASCII identifier text as Unicode characters.

## Optimization

- Preallocate one output string from the combined input byte lengths.
- Append `kind` and `path` directly without constructing an intermediate formatted string.
- Use an ASCII byte path for common identifiers and preserve one-underscore-per-character Unicode
  fallback semantics.

## Regression Contract

The shared `optimization_batch_20260826eg_` filter owns three Editor tests: ASCII/Unicode behavior,
single-output-allocation source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `EDITOR122_PROP_STATE_SUFFIX_SINGLE_ALLOCATION_BENCH_V1`, performs 8,192 transforms
per sample, reduces allocations per transform from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
