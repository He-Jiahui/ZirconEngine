# Editor250 Allocation-Free Missing-Field Display

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime304-editor250-performance-batch-20260829ae-v1`

## Problem

Incomplete asset type diagnostics joined all missing field labels into an intermediate `String`
before formatting the final error text. Every displayed registry error therefore allocated a
temporary list buffer in addition to the returned message.

## Optimization

- Write the diagnostic prefix directly to the destination formatter.
- Stream the first field and comma-separated remaining fields without `join`.
- Preserve the exact text for populated and empty missing-field lists.

## Regression Contract

The `optimization_batch_20260829ae_` Editor tests cover populated and empty diagnostics and guard
the allocation-free display source contract. The ignored paired release benchmark emits
`EDITOR250_ALLOCATION_FREE_MISSING_FIELD_DISPLAY_BENCH_V1`. It formats 100,000 eight-field
diagnostics per sample, reduces result buffers from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
