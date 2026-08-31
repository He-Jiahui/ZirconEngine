# Runtime258 Streaming Overview Query

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime258-editor204-performance-batch-20260826hl-v1`

## Problem

Material-management overview queries cloned the complete record vector before applying filters and
building the result. Selective queries therefore allocated and populated a temporary vector for
records that were immediately discarded.

## Optimization

- Stream cloned records from the borrowed overview slice into the query pipeline.
- Avoid allocating a complete intermediate vector before filtering and result collection.
- Preserve ownership of returned records and leave the source overview unchanged.

## Regression Contract

The `optimization_batch_20260826hl_` Runtime tests preserve owned clone semantics and source reuse;
enforce streaming iteration without a whole-vector clone; and provide an ignored paired release
benchmark emitting `RUNTIME258_STREAMING_OVERVIEW_QUERY_BENCH_V1`. It filters 16,384 records 64
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
