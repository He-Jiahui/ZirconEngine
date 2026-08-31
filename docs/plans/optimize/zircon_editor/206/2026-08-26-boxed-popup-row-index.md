# Editor206 Boxed Popup Row Index

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime260-editor206-performance-batch-20260826hn-v1`

## Problem

The pane hit-test index collected popup row numbers into a vector and then converted that vector
into `Arc<[usize]>`. The row index is never cloned or shared, so the conversion paid for a second
allocation and full row copy while retaining unnecessary atomic reference-count storage.

## Optimization

- Store popup row numbers in an exclusive `Box<[usize]>`.
- Convert the owned vector with `into_boxed_slice`, allowing its allocation to be reused.
- Preserve popup filtering, row order, and the existing borrowed-slice query contract.

## Regression Contract

The `optimization_batch_20260826hn_` Editor tests preserve popup row contents and owned allocation
reuse; enforce boxed non-shared storage; and provide an ignored paired release benchmark emitting
`EDITOR206_BOXED_POPUP_ROW_INDEX_BENCH_V1`. It packages 16,384 row indices 128 times per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
