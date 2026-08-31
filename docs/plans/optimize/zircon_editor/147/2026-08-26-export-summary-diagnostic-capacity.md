# Editor147 Export Summary Diagnostic Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime201-editor147-performance-batch-20260826ff-v1`

## Problem

The Editor export summary pane grew a temporary line vector from empty despite having two fixed
header lines, a known fatal-diagnostic count, and a six-row ordinary diagnostic display limit.

## Optimization

- Reserve the saturating total of two headers, all fatal diagnostics, and at most six ordinary
  diagnostics.
- Reuse the named six-row limit for both reservation and rendering while preserving output text,
  ordering, and truncation.

## Regression Contract

The `optimization_batch_20260826ff_` Editor tests cover output header text, 128 fatal rows, ordinary
diagnostic truncation at six, capacity math, source shape, and an ignored paired release benchmark
emitting `EDITOR147_EXPORT_SUMMARY_DIAGNOSTIC_CAPACITY_BENCH_V1`. It appends 256 lightweight lines
2,048 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
