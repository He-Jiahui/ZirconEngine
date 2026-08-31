# Editor234 Allocation-Free Patch Changes

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime288-editor234-performance-batch-20260828ip-v1`

## Problem

Every retained view patch comparison built a temporary heap Vec before immediately iterating at
most four changed properties. The common single-property update therefore allocated and freed a
collection for each changed control during incremental projection.

## Optimization

- Represent the four optional changes as a fixed array owned by an iterator.
- Construct String-valued mutations only when their property actually changed.
- Preserve selected, focused, surface, and text mutation order and values at the existing call site.

## Regression Contract

The `optimization_batch_20260828ip_` Editor tests prove exact mutation ordering and values and
guard the fixed-slot iterator source contract. The ignored paired release benchmark emits
`EDITOR234_ALLOCATION_FREE_PATCH_CHANGES_BENCH_V1`. It performs 200,000 single-property comparisons
per sample, removes 200,000 temporary heap Vecs, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
