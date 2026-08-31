# Runtime286 Matching Meta Identity

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime286-editor232-performance-batch-20260828in-v1`

## Problem

Loading an existing asset sidecar parsed its canonical resource locator, then cloned the scan URI
over it even when both locators were identical. Normal incremental imports therefore repeated the
path and optional label allocations before retaining the same identity.

## Optimization

- Compare the loaded locator with the canonical scan URI before replacing it.
- Preserve correction of stale locators and asset-kind refresh behavior.
- Avoid path and label allocation churn on the normal matching-identity path.

## Regression Contract

The `optimization_batch_20260828in_` Runtime tests prove matching path and label allocation identity,
stale-locator replacement, and the guarded source contract. The ignored paired release benchmark
emits `RUNTIME286_MATCHING_META_IDENTITY_BENCH_V1`. It performs 1,024 matching refreshes of a 64-KiB
locator per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
