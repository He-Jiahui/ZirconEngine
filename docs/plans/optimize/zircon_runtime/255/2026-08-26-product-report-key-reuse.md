# Runtime255 Product Report Key Reuse

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime255-editor201-performance-batch-20260826hi-v1`

## Problem

Every frame that published light-grid or virtual-geometry reports cloned the stable viewport-camera
history key before inserting into its `HashMap`. Existing camera rows are updated repeatedly, so
both product maps paid key-clone overhead even though neither map needed a new key allocation.

## Optimization

- Replace values for existing camera keys through a borrowed `get_mut` lookup.
- Share the same insertion helper across light-grid and virtual-geometry report maps.
- Clone the key only when a camera report is published for the first time.

## Regression Contract

The `optimization_batch_20260826hi_` Runtime tests preserve existing and new hash-map updates;
enforce both product maps using the borrowed-key helper; and provide an ignored paired release
benchmark emitting `RUNTIME255_PRODUCT_REPORT_KEY_REUSE_BENCH_V1`. It repeatedly updates a key with
a 32 KiB cloned payload and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
