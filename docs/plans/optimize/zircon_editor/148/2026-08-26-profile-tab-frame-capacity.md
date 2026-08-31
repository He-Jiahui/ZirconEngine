# Editor148 Profile Tab Frame Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime202-editor148-performance-batch-20260826fg-v1`

## Problem

Editor profiling artifact generation grew a tab-frame vector from empty while the retained model's
row count already provided a bounded output capacity.

## Optimization

- Reserve the tab model row count before projecting visible frames.
- Preserve missing-row handling, invisible-frame filtering, model order, translated frames, tab
  metadata, and close-frame projection.

## Regression Contract

The `optimization_batch_20260826fg_` Editor tests cover 256 visible model rows, id order, origin
translation, close-frame projection, final capacity, source shape, and an ignored paired release
benchmark emitting `EDITOR148_PROFILE_TAB_FRAME_CAPACITY_BENCH_V1`. It appends 256 lightweight
frames 2,048 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
