# Editor236 Direct Fixed-Root Authority

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime290-editor236-performance-batch-20260828ir-v1`

## Problem

Editor source-authority classification converted each fixed resource root into a synthetic locator,
allocated its text, parsed and normalized it, then discarded the locator after reading only its
scheme. The fixed roots have unambiguous source kinds and need no path validation.

## Optimization

- Match `res://`, `lib://`, `builtin://`, and `mem://` directly to their source kinds.
- Return the existing policy-aware authority without constructing a synthetic locator.
- Keep package-root validation and every non-root target on the original `ResourceLocator` path.

## Regression Contract

The `optimization_batch_20260828ir_` Editor tests compare all four fixed-root results with the legacy
synthetic-locator path and guard the direct-match ordering. The ignored paired release benchmark
emits `EDITOR236_DIRECT_FIXED_ROOT_AUTHORITY_BENCH_V1`. It performs 100,000 round-robin fixed-root
classifications per sample, reduces complete locator parses from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
