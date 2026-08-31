# Runtime214 View Visible Index Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime214-editor160-performance-batch-20260826fs-v1`

## Problem

Runtime custom-target and shadow visibility views appended primitive indices into vectors grown
from empty even though frame relevance length was a constant-time upper bound for both filtering
loops.

## Optimization

- Reserve the frame relevance count before both custom-target and shadow visible-index loops.
- Preserve layer, relevance, frustum and shadow-caster filtering, culling statistics, index order,
  and empty output; culled primitives only leave unused capacity.

## Regression Contract

The `optimization_batch_20260826fs_` Runtime tests verify full primitive-index capacity and order,
enforce both production reservation sites, and provide an ignored paired release benchmark emitting
`RUNTIME214_VIEW_VISIBLE_INDEX_CAPACITY_BENCH_V1`. It fills 128 vectors of 4,096 visibility entries
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
