# Editor94 Menu Action Label Single Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime148-editor94-performance-batch-20260826de-v1`

## Problem

Component-showcase context menu actions converted every underscore-delimited segment into a
separate `String`, collected those strings into a `Vec`, then joined them into the final label.
Long action IDs therefore created segment-count-proportional temporary allocations.

## Optimization

- Reserve one output buffer from the encoded action length.
- Append the ASCII-uppercased first character, remaining segment, and separators directly.
- Preserve empty-segment filtering, menu separators, disabled items, ordinary authored labels, and
  mixed Unicode segment behavior.

## Regression Contract

The shared `optimization_batch_20260826de_` filter owns three Editor tests: protocol behavior,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR94_MENU_ACTION_LABEL_SINGLE_BUFFER_BENCH_V1`, builds 131,072 seven-segment labels per
sample, records allocations from nine to one per label, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
