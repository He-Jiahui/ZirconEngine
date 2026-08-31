# Editor139 Invalidation Summary Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime193-editor139-performance-batch-20260826ex-v1`

## Problem

Host invalidation summaries can append up to eleven known reason names but built the temporary
name vector through repeated growth.

## Optimization

- Allocate once to the fixed eleven-reason maximum.
- Preserve reason order, separator formatting, and the empty mask's allocation-free `none` path.

## Regression Contract

The `optimization_batch_20260826ex_` Editor tests cover all eleven reason bits and summary order,
source shape, and an ignored paired release benchmark emitting
`EDITOR139_INVALIDATION_SUMMARY_CAPACITY_BENCH_V1`. It writes eleven names 47,663 times per sample
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
