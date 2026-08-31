# Runtime227 Descriptor Target Mode Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime227-editor173-performance-batch-20260826gf-v1`

## Problem

Runtime plugin descriptor validation allocated a temporary Vec for every target-mode list solely to
check whether each mode had appeared earlier in the same input.

## Optimization

- Compare each target mode with the already-visited input prefix instead of materializing a seen
  vector.
- Preserve the empty-list diagnostic and emit one duplicate diagnostic for every repeated
  occurrence, including a third or later declaration.
- Preserve target-mode order and diagnostic text while eliminating the validator's temporary heap
  allocation.

## Regression Contract

The `optimization_batch_20260826gf_` Runtime tests cover missing and repeated target-mode diagnostics
and enforce the input-prefix source contract, and provide an ignored paired release benchmark
emitting `RUNTIME227_DESCRIPTOR_TARGET_MODE_ALLOCATION_BENCH_V1`. It validates 262,144 four-mode
lists per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
