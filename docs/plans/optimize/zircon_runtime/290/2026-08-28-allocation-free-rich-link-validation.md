# Runtime290 Allocation-Free Rich-Link Validation

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime290-editor236-performance-batch-20260828ir-v1`

## Problem

Every retained UI rich-link activation constructed and normalized a complete `ResourceLocator` only
to check whether its scheme was allowed. The locator was discarded immediately, so canonical links
paid for path copies, component storage, and normalization output on every activation.

## Optimization

- Split the scheme and optional label as borrowed slices.
- Validate canonical forward-slash paths with `Path::components` and an allocation-free depth count.
- Preserve package-root isolation by validating the package ID and package-relative path separately.
- Retain the original locator parser as the exact compatibility fallback for backslash paths.

## Regression Contract

The `optimization_batch_20260828ir_` Runtime tests compare accepted and rejected targets against the
original locator semantics and guard the compatibility fallback. The ignored paired release benchmark
emits `RUNTIME290_ALLOCATION_FREE_RICH_LINK_VALIDATION_BENCH_V1`. It performs 100,000 validations
of an 84-byte canonical target per sample, reduces complete locator parses from one to zero, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
