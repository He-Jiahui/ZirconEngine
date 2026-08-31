# Runtime152 Enum Label Single Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime152-editor98-performance-batch-20260826di-v1`

## Problem

Material component option registration allocated one `String` per underscore-delimited segment, a
temporary `Vec<String>`, and the required joined result while converting enum values to labels.
Catalog construction repeats that work across every option descriptor.

## Optimization

- Reserve the final label buffer from the source byte length.
- Replace underscores with spaces and append capitalized segments directly into that buffer.
- Preserve empty segments, leading/trailing underscores, ASCII capitalization, and non-ASCII text.

## Regression Contract

The shared `optimization_batch_20260826di_` filter owns three Runtime tests: output behavior,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME152_ENUM_LABEL_SINGLE_BUFFER_BENCH_V1`, renders 131,072 seven-segment labels per sample,
records allocations per label from at least nine to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
