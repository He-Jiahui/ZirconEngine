# Editor141 Binding Detail Row Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime195-editor141-performance-batch-20260826ez-v1`

## Problem

UI asset binding detail projection can emit five known rows but built its temporary row vector
through repeated growth.

## Optimization

- Allocate once to the named five-row upper bound.
- Preserve field order, visibility rules, editability, action identifiers, and control identifiers.

## Regression Contract

The `optimization_batch_20260826ez_` Editor tests cover all five editable binding rows and their
order, source shape, and an ignored paired release benchmark emitting
`EDITOR141_BINDING_DETAIL_ROW_CAPACITY_BENCH_V1`. It writes five lightweight row entries 104,858
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
