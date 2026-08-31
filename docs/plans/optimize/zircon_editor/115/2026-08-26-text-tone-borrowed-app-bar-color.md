# Editor115 Text Tone Borrowed App Bar Color

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime169-editor115-performance-batch-20260826dz-v1`

## Problem

Retained AppBar text-tone projection cloned the color attribute only to compare it against
`inherit` and `transparent`. The temporary owned string did not enter the final model.

## Optimization

- Borrow AppBar color directly from TOML attributes for text-tone classification.
- Return the static `primary` default without allocation.
- Preserve explicit text-tone ownership and all final tone values.

## Regression Contract

The shared `optimization_batch_20260826dz_` filter owns three Editor tests: tone behavior, borrowed
pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR115_TEXT_TONE_BORROWED_APP_BAR_COLOR_BENCH_V1`, performs 524,288 lookups per sample, reduces
lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
