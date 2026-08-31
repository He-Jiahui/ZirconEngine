# Runtime260 Owned Issue View Sort

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime260-editor206-performance-batch-20260826hn-v1`

## Problem

Each sorted material-issue constructor first built an owned filtered view and then called the
borrowing `sorted` API. That API deep-cloned every overview record into a second vector before
sorting, after which the first record vector and its material-id projection were discarded.

## Optimization

- Route all three owned constructors through a consuming in-place sort path.
- Reuse the freshly allocated record vector and rebuild only the order-dependent material-id list.
- Preserve the public borrowing `sorted` API and all issue-filter and ordering semantics.

## Regression Contract

The `optimization_batch_20260826hn_` Runtime tests preserve sorting and vector-allocation identity;
enforce consuming sort use in all three owned constructors; and provide an ignored paired release
benchmark emitting `RUNTIME260_OWNED_ISSUE_VIEW_SORT_BENCH_V1`. It sorts 8,192 named records 32
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
