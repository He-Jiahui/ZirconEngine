# Runtime262 Owned Overview Sort

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime262-editor208-performance-batch-20260826hp-v1`

## Problem

Record-set sorted overview construction first built an owned overview and then called its borrowing
`sorted` API. That API deep-cloned every overview record into a second vector before sorting and
discarded the first owned vector immediately afterward.

## Optimization

- Sort the freshly built overview record vector in place.
- Rebuild the order-dependent status and issue indexes from the sorted records.
- Preserve summary counts, source record-set ownership, and the public borrowing overview API.

## Regression Contract

The `optimization_batch_20260826hp_` Runtime tests preserve sorting and vector-allocation identity;
enforce owned overview sorting plus both index rebuilds; and provide an ignored paired release
benchmark emitting `RUNTIME262_OWNED_OVERVIEW_SORT_BENCH_V1`. It sorts 8,192 named records 32 times
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
