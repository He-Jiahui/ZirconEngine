# Runtime197 UI Reflection Diff Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime197-editor143-performance-batch-20260826fb-v1`

## Problem

UI reflection diff assembly appended changed and removed node IDs to growth-driven vectors even
though both exact counts can be derived from the two immutable snapshots.

## Optimization

- Precompute changed and removed counts with the same comparison predicates as the assembly loops.
- Allocate both result vectors exactly while preserving BTreeMap order and diff semantics.

## Regression Contract

The `optimization_batch_20260826fb_` Runtime tests cover 256 changed and 128 removed nodes,
identifier order and exact capacity math, source shape, and an ignored paired release benchmark
emitting `RUNTIME197_UI_REFLECTION_DIFF_CAPACITY_BENCH_V1`. It writes 384 lightweight node IDs
1,366 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
