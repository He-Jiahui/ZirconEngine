# Editor239 Lazy Typography Default Weights

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime293-editor239-performance-batch-20260828iu-v1`

## Problem

Projecting editor typography eagerly constructed the full workbench-default token object only to
read three fallback weights. Valid configured weights are the normal path, so every projection
unnecessarily allocated the default UI, code, and tab-label family strings.

## Optimization

- Validate all three numeric weights before constructing fallback typography tokens.
- Return configured values directly when every weight is valid.
- Create the existing workbench defaults only for an invalid-weight fallback.
- Preserve independent fallback behavior for every invalid field.

## Regression Contract

The `optimization_batch_20260828iu_` Editor tests cover valid direct projection, invalid default
fallback, and the lazy-default source contract. The ignored paired release benchmark emits
`EDITOR239_LAZY_TYPOGRAPHY_DEFAULT_WEIGHTS_BENCH_V1`. It performs 100,000 projections per sample,
reduces default-string allocations on the valid path from three to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
