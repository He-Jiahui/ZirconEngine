# Editor157 Inspector Prop State Row Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime211-editor157-performance-batch-20260826fp-v1`

## Problem

The Editor asset inspector projected selected-node property and state maps into a row vector grown
from empty even though both top-level map lengths were available without traversal.

## Optimization

- Reserve the saturating sum of property and state entry counts before recursive row projection.
- Use that sum as a guaranteed lower bound for nested tables and an exact bound for flat maps while
  preserving property-before-state order, paths, display strings, and values.

## Regression Contract

The `optimization_batch_20260826fp_` Editor tests project 128 properties and 128 states, verify row
order and capacity, enforce the production source shape, and provide an ignored paired release
benchmark emitting `EDITOR157_INSPECTOR_PROP_STATE_ROW_CAPACITY_BENCH_V1`. It fills 128 vectors of
4,096 pending-row-sized fixtures per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
