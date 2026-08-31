# Editor249 Single-Buffer Layout Preset Sanitizer

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime303-editor249-performance-batch-20260829ad-v1`

## Problem

Layout preset names were first mapped into a complete temporary `String`, then trimmed and copied
into a second allocation. Every saved preset name paid for two result buffers even when the first
buffer already contained the final bytes plus removable edge separators.

## Optimization

- Reserve one result buffer from the input length.
- Delay separator writes until a following valid character proves they are internal.
- Drop leading and trailing separators without a second string allocation.

## Regression Contract

The `optimization_batch_20260829ad_` Editor tests cover edge trimming, Unicode replacement,
internal separator runs, valid names, and the empty fallback and guard the single-buffer source
contract. The ignored paired release benchmark emits
`EDITOR249_SINGLE_BUFFER_LAYOUT_PRESET_SANITIZER_BENCH_V1`. It sanitizes 100,000 names per sample,
reduces result allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
