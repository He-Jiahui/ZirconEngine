# Editor174 Region Frame Scratch Allocation

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime228-editor174-performance-batch-20260826gh-v1`

## Problem

Editor workbench region-frame resolution allocated separate temporary vectors for visible region IDs
and width constraints before invoking the axis solver, despite there being only four visibility
combinations and at most three row regions.

## Optimization

- Select fixed region and constraint slices on the stack for each left/right visibility combination.
- Retain the solver result and compacted output vectors required by downstream ownership while
  removing the two scratch input allocations.
- Preserve left-document-right order, solver inputs, compact side balancing, frame placement, and
  all visibility combinations.

## Regression Contract

The `optimization_batch_20260826gh_` Editor tests cover region order for all four visibility
combinations and enforce stack-slice solver inputs, and provide an ignored paired release benchmark
emitting `EDITOR174_REGION_FRAME_SCRATCH_ALLOCATION_BENCH_V1`. It resolves 262,144 scratch input sets
per sample with the shared output allocation retained and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
