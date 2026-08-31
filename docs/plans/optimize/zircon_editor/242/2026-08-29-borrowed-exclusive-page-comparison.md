# Editor242 Borrowed Exclusive-Page Comparison

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime296-editor242-performance-batch-20260829w-v1`

## Problem

Focusing an exclusive activity-window page cloned its `MainPageId` solely to compare it with the
currently active page. A real page change then cloned the same identifier again to transfer
ownership into the layout, so the common comparison path paid an avoidable string allocation.

## Optimization

- Compare the active and candidate page identifiers through borrowed references.
- Retain the ownership clone only when the active page actually changes.
- Preserve the existing changed flag, assignment, and early-return behavior.

## Regression Contract

The `optimization_batch_20260829w_` Editor tests cover equal and different page identifiers and
guard the borrowed comparison in the exclusive-page focus branch. The ignored paired release
benchmark emits `EDITOR242_BORROWED_EXCLUSIVE_PAGE_COMPARISON_BENCH_V1`. It performs 100,000
alternating equal/different comparisons per sample over a 97-byte identifier, reduces page-ID
allocations per comparison from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
