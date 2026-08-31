# Editor207 Shared Vec Grid Storage

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime261-editor207-performance-batch-20260826ho-v1`

## Problem

Sample-grid construction owned its x ticks, y ticks, and points in vectors, then converted each
vector into `Arc<[T]>`. Those conversions allocated shared slices and copied every element even
though generation cloning only needs shared ownership and all consumers already borrow slices.

## Optimization

- Store the three owned arrays as `Arc<Vec<T>>` so their element allocations move unchanged.
- Preserve cheap shared generation clones through Arc ownership.
- Keep hashing and public grid access on borrowed slices via `as_slice`.

## Regression Contract

The `optimization_batch_20260826ho_` Editor tests preserve element-allocation identity and Arc clone
sharing; enforce shared Vec storage for all three grid arrays; and provide an ignored paired release
benchmark emitting `EDITOR207_SHARED_VEC_GRID_STORAGE_BENCH_V1`. It packages 16,384 items 128 times
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
