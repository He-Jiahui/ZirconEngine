# Editor158 Menu Reflection Leaf Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime212-editor158-performance-batch-20260826fq-v1`

## Problem

Editor workbench reflection recursively flattened menu leaves into a vector grown from empty even
though the same tree semantics could determine the exact emitted leaf count before projection.

## Optimization

- Count terminal menu items recursively with saturating addition and reserve the exact reflection
  row count before the existing depth-first projection.
- Preserve menu order, nested leaf order, empty-branch behavior, bindings, labels, shortcuts,
  enabled flags, operation paths, and route state.

## Regression Contract

The `optimization_batch_20260826fq_` Editor tests project a nested four-leaf menu, verify exact leaf
count, order, capacity, and production source shape, and provide an ignored paired release benchmark
emitting `EDITOR158_MENU_REFLECTION_LEAF_CAPACITY_BENCH_V1`. It fills 128 vectors of 4,096
reflection-row-sized fixtures per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
