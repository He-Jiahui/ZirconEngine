---
title: Runtime473 Reserved Bubble Route Capacity
category: zircon_runtime
report_id: Runtime473-reserved-bubble-route-capacity-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime473 Reserved Bubble Route Capacity

Runtime UI bubble-route construction now reserves capacity for the common first 16 ancestors,
bounded by the tree's current node count. The target-to-root order, parent traversal, missing-node
error, and support for routes deeper than 16 nodes are unchanged. The cap prevents a large UI tree
from causing excessive reservation when the dispatched node is near a root.

This removes the repeated growth allocations produced by starting a common-depth input route from
an empty vector. Regression coverage constructs a 16-node tree and checks route order, length, and
capacity. The ignored Windows Release benchmark emits
`RUNTIME473_RESERVED_BUBBLE_ROUTE_BENCH_V1` over 17 alternating paired samples and 32,768 routes
per sample. A 16-node route falls from three growth allocations to zero after the initial exact
reservation. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.60` (at least 40% lower
P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime473 is prepared with Editor403 under request
`runtime473-editor403-performance-batch-20260831fq-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
