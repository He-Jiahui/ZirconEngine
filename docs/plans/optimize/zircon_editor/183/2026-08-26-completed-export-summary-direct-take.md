# Editor183 Completed Export Summary Direct Take

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime237-editor183-performance-batch-20260826gq-v1`

## Problem

Every export queue poll drained completed summaries from a VecDeque into a newly allocated Vec,
moving every summary while retaining the now-empty source allocation in the queue.

## Optimization

- Take ownership of the completed VecDeque in constant time and replace it with an empty queue.
- Convert the owned VecDeque into the returned Vec while preserving logical order, including wrapped storage.
- Keep terminal-ticket append behavior and the public polling result type unchanged.

## Regression Contract

The `optimization_batch_20260826gq_` Editor tests cover wrapped-order ownership transfer, enforce the
direct-take source contract, and provide an ignored paired release benchmark emitting
`EDITOR183_COMPLETED_EXPORT_SUMMARY_DIRECT_TAKE_BENCH_V1`. It transfers 128 batches of 1,024 summaries
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
