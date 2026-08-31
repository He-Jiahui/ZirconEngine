# Editor100 Viewport Field Label Single Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime154-editor100-performance-batch-20260826dk-v1`

## Problem

Scene viewport inspector projection allocated one string per underscore-delimited field-name
segment, collected those strings into a temporary vector, and allocated the final title-cased
label. Inspector refresh repeats the conversion across reflected fields.

## Optimization

- Reserve the final title buffer from the source byte length.
- Append non-empty capitalized segments and separators directly into that buffer.
- Preserve empty-segment filtering, ASCII capitalization, and non-ASCII text.

## Regression Contract

The shared `optimization_batch_20260826dk_` filter owns three Editor tests: label behavior,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR100_VIEWPORT_FIELD_LABEL_SINGLE_BUFFER_BENCH_V1`, renders 131,072 seven-segment labels per
sample, records allocations per label from at least nine to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
