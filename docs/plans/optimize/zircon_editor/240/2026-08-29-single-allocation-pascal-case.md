# Editor240 Single-Allocation Pascal Case

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime294-editor240-performance-batch-20260829iv-v1`

## Problem

Every projected surface variant converted its first character into a temporary `String` and then
grew that allocation while appending the remaining text. Badge, chip, alert, dialog, and general
component projection invoke this helper repeatedly while composing variant tokens.

## Optimization

- Reserve the exact input byte length once for non-empty values.
- Push the ASCII-uppercased first character directly into the result.
- Append the remaining borrowed slice without an intermediate string.
- Preserve empty, non-letter, already-uppercase, and non-ASCII behavior.

## Regression Contract

The `optimization_batch_20260829iv_` Editor tests cover ASCII, numeric, empty, and Unicode inputs
and guard the single-output allocation shape. The ignored paired release benchmark emits
`EDITOR240_SINGLE_ALLOCATION_PASCAL_CASE_BENCH_V1`. It performs 100,000 conversions of a 70-byte
value per sample, reduces allocations per conversion from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
