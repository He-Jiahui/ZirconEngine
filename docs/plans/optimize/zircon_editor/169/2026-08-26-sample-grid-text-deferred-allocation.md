# Editor169 Sample Grid Text Deferred Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime223-editor169-performance-batch-20260826gb-v1`

## Problem

Editor sample-grid painting cloned borrowed tick and axis labels before its shared text helper rejected
empty or invalid frames, allocating strings for labels that could not produce paint commands.

## Optimization

- Accept borrowed or owned text through `Cow<str>` in the shared sample-grid text helper.
- Pass tick and axis labels by reference and convert to owned command text only after all gates pass.
- Preserve the selected-point caller's owned-string move, text order, geometry, measurement, color,
  clipping, typography, opacity, and successful command text.

## Regression Contract

The `optimization_batch_20260826gb_` Editor tests cover empty-label filtering, Cow ownership order,
and removal of eager tick/axis clones, and provide an ignored paired release benchmark emitting
`EDITOR169_SAMPLE_GRID_TEXT_DEFERRED_ALLOCATION_BENCH_V1`. It rejects 8,192 labels of 4,096 bytes per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
