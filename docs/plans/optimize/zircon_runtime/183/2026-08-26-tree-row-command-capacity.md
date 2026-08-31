# Runtime183 Tree Row Command Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime183-editor129-performance-batch-20260826en-v1`

## Problem

Runtime tree-row rendering started every command list empty even though a row always emits four
icons and can add one surface, one label, and one indentation command per tree depth. Deeper rows
therefore repeatedly grew and moved the render-command buffer. The same `tree_depth` metadata was
also decoded once for indentation commands and again for disclosure placement.

## Optimization

- Decode the non-negative tree depth once and reuse it for indentation and disclosure geometry.
- Allocate the render-command vector to the exact `depth + 6` upper bound before emission.
- Preserve command order, z values, explicit `tree_indent_px`, optional surface/label behavior, and
  the existing four-command minimum.

## Regression Contract

The shared `optimization_batch_20260826en_` filter owns three Runtime tests: maximum-row behavior,
capacity/depth source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME183_TREE_ROW_COMMAND_CAPACITY_BENCH_V1`, builds 262 real `UiRenderCommand` values 1,024
times per sample, replaces growth-driven allocation with one exact allocation, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
