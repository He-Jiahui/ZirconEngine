# Editor132 Layout Detail Row Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime186-editor132-performance-batch-20260826eq-v1`

## Problem

Layout inspector conversion can emit ten rows but grew its vector repeatedly despite all row
visibility conditions being available before string/control-ID materialization.

## Optimization

- Count the ten non-empty Layout values and allocate exactly once.
- Preserve row order/content and the empty inspector's zero allocation.

## Regression Contract

The `optimization_batch_20260826eq_` Editor tests cover full/empty behavior, source shape, and an
ignored paired release benchmark emitting `EDITOR132_LAYOUT_DETAIL_ROW_CAPACITY_BENCH_V1`. It
writes ten real rows 65,536 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
