# Runtime305 Direct Profile Registration Forwarding

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime305-editor251-performance-batch-20260829af-v1`

## Problem

Runtime profile assembly collected borrowed plugin registration iterators into temporary vectors,
then immediately converted those vectors back into copied reference iterators for the lower
assembly owner. The plugin path allocated one redundant vector and the feature path allocated two.

## Optimization

- Forward plugin registration iterators directly to the lower assembly owner.
- Forward plugin and feature registration iterators without intermediate borrowed-reference lists.
- Preserve the lower owner's filtering, owned snapshots, availability checks, and ordering.

## Regression Contract

The `optimization_batch_20260829af_` Runtime tests cover ordering, empty input, and both production
wrapper source contracts. The ignored paired release benchmark emits
`RUNTIME305_DIRECT_PROFILE_REGISTRATION_FORWARDING_BENCH_V1`. It forwards an eight-registration
profile 200,000 times per sample, reduces outer vector allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
