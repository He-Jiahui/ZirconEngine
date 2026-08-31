# Runtime292 Canonical Builtin-Locator Match

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime292-editor238-performance-batch-20260828it-v1`

## Problem

Restoring one builtin asset parsed and normalized every builtin locator candidate before comparing it
with the registry locator. Builtin inventory keys are canonical constants, so every unsuccessful
candidate paid for path copies, component storage, normalization, and final locator ownership.

## Optimization

- Strip the fixed `builtin://` prefix as a borrowed slice.
- Split the optional label without allocating and compare path and label directly.
- Keep the helper private to the builtin inventory scan, where candidate canonicality is guaranteed.
- Avoid depending on the separately modified runtime-interface locator implementation.

## Regression Contract

The `optimization_batch_20260828it_` Runtime tests compare exact and mismatched canonical candidates
against the legacy parser and guard the allocation-free scan. The ignored paired release benchmark
emits `RUNTIME292_CANONICAL_BUILTIN_LOCATOR_MATCH_BENCH_V1`. It performs 100,000 matches of an
81-byte locator per sample, reduces locator parses from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
