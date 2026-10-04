---
title: Runtime625 Preallocated Frame Batching Collections
category: zircon_runtime
report_id: Runtime625-preallocated-frame-batching-collections-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime625 Preallocated Frame Batching Collections

Visibility frame batching now uses the already-materialized mesh index count to reserve three
entity membership sets and the primitive relevance, BVH instance, and history output vectors.
The old path grew all six collections from zero on every frame.

Stable mesh traversal, mobility partitioning, ordered batch keys, entity membership semantics, and
the downstream sorted projections remain unchanged. Existing visibility tests still cover the
frame result, while focused source coverage locks all six preallocations.

The ignored Windows Release benchmark emits `RUNTIME625_PREALLOCATED_FRAME_BATCHING_BENCH_V1` over
17 alternating sample pairs with 65,536 unique entities. The gate requires preallocated collection
P95 to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime625/Editor625 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime625 is prepared with Editor625 under request
`runtime625-editor625-frame-project-membership-performance-20260901io-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
