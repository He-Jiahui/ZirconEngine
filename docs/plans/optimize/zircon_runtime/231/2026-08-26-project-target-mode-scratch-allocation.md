# Runtime231 Project Target Mode Scratch Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime231-editor177-performance-batch-20260826gk-v1`

## Problem

Project-manifest target-mode validation and sanitization allocated separate temporary vectors while
processing rows whose values come from an exhaustive three-variant domain.

## Optimization

- Detect duplicate diagnostics against the already visited input prefix without scratch storage.
- Deduplicate in place with a three-bit ClientRuntime, ServerRuntime, and EditorHost membership mask.
- Preserve every duplicate diagnostic, first-occurrence order, target filtering, and in-place output.

## Regression Contract

The `optimization_batch_20260826gk_` Runtime tests cover duplicate diagnostic multiplicity and
stable first-occurrence order, enforce prefix and bitset scratch handling, and provide an ignored
paired release benchmark emitting `RUNTIME231_PROJECT_TARGET_MODE_SCRATCH_ALLOCATION_BENCH_V1`.
It processes 262,144 six-mode rows per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
