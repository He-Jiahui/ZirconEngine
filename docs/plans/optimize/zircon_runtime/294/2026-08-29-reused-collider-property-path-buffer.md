# Runtime294 Reused Collider Property-Path Buffer

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime294-editor240-performance-batch-20260829iv-v1`

## Problem

Collider property enumeration formatted a new owned path for every emitted field. Compound shapes
also formatted child prefixes and recursively allocated another path per descendant, so editor and
reflection scans paid allocation cost proportional to the number of projected properties.

## Optimization

- Allocate one path buffer for the complete collider traversal.
- Pre-size that buffer from the longest path reachable through the actual shape tree.
- Append suffixes in place and truncate to the previous checkpoint after each visitor call.
- Reuse the same buffer through compound-shape recursion and indexed point paths.

## Regression Contract

The `optimization_batch_20260829iv_` Runtime tests verify recursive compound paths and guard the
single-buffer traversal contract. The ignored paired release benchmark emits
`RUNTIME294_REUSED_COLLIDER_PROPERTY_PATH_BUFFER_BENCH_V1`. It performs 5,000 traversals of a
64-point convex hull per sample, reduces path allocations per traversal from 66 to one, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
