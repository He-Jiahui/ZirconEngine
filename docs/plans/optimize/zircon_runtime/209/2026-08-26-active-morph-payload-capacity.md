# Runtime209 Active Morph Payload Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime209-editor155-performance-batch-20260826fn-v1`

## Problem

Per-draw morph payload construction grew delta, current-weight, and previous-weight vectors from
empty even though active targets could be bounded from current and previous frame weights before
visiting vertex attributes.

## Optimization

- Count the targets whose current or previous weight is nonzero, reserve that count for both weight
  vectors, and reserve `active targets × vertices × 4` delta rows with saturating arithmetic.
- Preserve previous-only velocity targets, target order, invalid attribute filtering, zero-weight
  skipping, and all position/normal/tangent/color row semantics.

## Regression Contract

The `optimization_batch_20260826fn_` Runtime tests build a real 64-vertex, 64-target mesh with 33
active targets including one previous-only target, verify row and weight order plus all three final
capacities, enforce the production source shape, and provide an ignored paired release benchmark
emitting `RUNTIME209_ACTIVE_MORPH_PAYLOAD_CAPACITY_BENCH_V1`. It fills 8,448 delta rows and two
33-element weight vectors 256 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
