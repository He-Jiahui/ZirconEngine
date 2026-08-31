# Editor144 Activity Property Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime198-editor144-performance-batch-20260826fc-v1`

## Problem

Editor activity reflection initialized three core property descriptors and then grew the vector
again while appending an already counted custom property map.

## Optimization

- Reserve the exact `3 + custom_property_count` capacity with saturating arithmetic.
- Preserve core property order, custom BTreeMap order, inferred value types, and node state flags.

## Regression Contract

The `optimization_batch_20260826fc_` Editor tests cover three core plus 253 custom properties,
final node contents and exact capacity math, source shape, and an ignored paired release benchmark
emitting `EDITOR144_ACTIVITY_PROPERTY_CAPACITY_BENCH_V1`. It writes 256 lightweight properties
2,048 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
