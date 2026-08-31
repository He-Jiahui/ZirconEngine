# Runtime196 Active Feature Selection Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime196-editor142-performance-batch-20260826fa-v1`

## Problem

Runtime plugin feature resolution scanned project selections for enabled features but appended the
result to a growth-driven vector despite the same predicate being available before assembly.

## Optimization

- Count enabled feature rows before projection and reserve the exact active output capacity.
- Preserve owner identity, feature borrowing, selection order, and the enabled predicate.

## Regression Contract

The `optimization_batch_20260826fa_` Runtime tests cover 256 mixed enabled/disabled features,
active ordering and exact capacity math, source shape, and an ignored paired release benchmark
emitting `RUNTIME196_ACTIVE_FEATURE_SELECTION_CAPACITY_BENCH_V1`. It writes 128 active entries
4,096 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
