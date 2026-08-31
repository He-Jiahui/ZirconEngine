# Runtime314 Recent Material Diagnostic Dedup

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime314-editor259-performance-batch-20260829an-v1`

## Problem

Material readiness collectors performed a full linear duplicate scan for every validation error,
fallback usage, and diagnostic, including the common case where a producer repeated the most
recently emitted issue. Long issue lists made that repeat path linear in accumulated history.

## Optimization

- Compare the most recent item before falling back to the full duplicate scan.
- Share the same helper across validation errors, fallback usages, and diagnostics.
- Preserve first-occurrence order and the existing non-adjacent duplicate behavior.

## Regression Contract

The `optimization_batch_20260829an_` Runtime tests cover recent, earlier, and new items and guard
all three production call sites. The ignored paired release benchmark emits
`RUNTIME314_RECENT_MATERIAL_DIAGNOSTIC_DEDUP_BENCH_V1`. It performs 5,000 duplicate checks per
sample against 512 existing items, changes 512 comparisons per check to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
