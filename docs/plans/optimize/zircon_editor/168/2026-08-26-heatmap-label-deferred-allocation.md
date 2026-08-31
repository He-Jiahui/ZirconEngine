# Editor168 Heatmap Label Deferred Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime222-editor168-performance-batch-20260826ga-v1`

## Problem

Editor weight-heatmap legend painting cloned both high and low labels before checking whether the
label was non-empty and its computed frame was drawable, allocating text for collapsed legends.

## Optimization

- Pass high and low legend labels as borrowed strings through validation and measurement.
- Allocate owned command text only after the empty-label and geometry gates succeed.
- Preserve label order, line placement, measured width, typography, clipping, color, opacity, and
  successful command text.

## Regression Contract

The `optimization_batch_20260826ga_` Editor tests cover empty-label filtering and allocation order,
and provide an ignored paired release benchmark emitting
`EDITOR168_HEATMAP_LABEL_DEFERRED_ALLOCATION_BENCH_V1`. It rejects 8,192 labels of 4,096 bytes per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
