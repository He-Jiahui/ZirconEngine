# Editor150 External Effect Asset Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime204-editor150-performance-batch-20260826fi-v1`

## Problem

Editor UI asset undo and redo each duplicated an external-effect loop and grew the affected asset
id vector from empty despite the replay effect count being exact.

## Optimization

- Route both undo and redo through one private generic batch applicator that reserves the effect
  count before collecting affected asset ids.
- Preserve effect order, immediate error propagation, project lookup timing, workspace refresh,
  import hydration, and instance synchronization.

## Regression Contract

The `optimization_batch_20260826fi_` Editor tests cover 256 ordered results, exact result capacity,
both production call sites, first-error short circuiting, and an ignored paired release benchmark
emitting `EDITOR150_EXTERNAL_EFFECT_ASSET_CAPACITY_BENCH_V1`. It applies 256 lightweight effects
2,048 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
