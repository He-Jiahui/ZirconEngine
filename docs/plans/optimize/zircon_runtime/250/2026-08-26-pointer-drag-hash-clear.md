# Runtime250 Pointer Drag Hash Clear

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime250-editor196-performance-batch-20260826hd-v1`

## Problem

Clearing pointer-drag state for removed UI nodes retained every active drag by linearly searching the
complete removed-node slice. Large tree invalidations therefore performed
`active_drag_count * removed_node_count` node-id comparisons.

## Optimization

- Preserve slice membership checks below 64 removed nodes to avoid small-batch allocation.
- Hash large removed-node sets once before retaining active pointer drags.
- Keep the existing ownership and retained-order semantics of the pointer-drag map.

## Regression Contract

The `optimization_batch_20260826hd_` Runtime tests preserve removed and retained drag ownership,
enforce the thresholded hash index, and provide an ignored paired release benchmark emitting
`RUNTIME250_POINTER_DRAG_HASH_CLEAR_BENCH_V1`. It clears 1,024 node ids from 2,048 active drags and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
