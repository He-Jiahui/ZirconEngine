# Editor170 Timeline Text Deferred Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime224-editor170-performance-batch-20260826gc-v1`

## Problem

Editor timeline painting cloned borrowed tick and track labels before the shared text helper rejected
empty or invalid frames, allocating strings for collapsed ruler and track geometry.

## Optimization

- Accept borrowed or owned timeline text through `Cow<str>` and borrow tick/track labels.
- Convert to owned command text only after the empty and finite positive-frame gates pass.
- Preserve footer String moves, tick/track/footer order, geometry, measurements, colors, typography,
  clipping, opacity, and successful command text.

## Regression Contract

The `optimization_batch_20260826gc_` Editor tests cover empty filtering, Cow ownership order, and
removal of eager tick/track clones, and provide an ignored paired release benchmark emitting
`EDITOR170_TIMELINE_TEXT_DEFERRED_ALLOCATION_BENCH_V1`. It rejects 8,192 labels of 4,096 bytes per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
