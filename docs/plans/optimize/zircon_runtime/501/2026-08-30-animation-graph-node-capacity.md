---
title: Runtime501 Animation Graph Node Capacity
category: zircon_runtime
report_id: Runtime501-animation-graph-node-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime501 Animation Graph Node Capacity

Animation graph compilation previously grew its non-output node and output-source vectors from
zero while scanning an already materialized node array. The collector now counts output variants,
derives the complementary node count, and reserves both result partitions exactly before the
stable source-order pass. Duplicate diagnostics, index assignment, and output order are unchanged.

The focused regression checks result order, indexes, and exact final capacities. The ignored
Windows Release benchmark emits `RUNTIME501_ANIMATION_GRAPH_NODE_CAPACITY_BENCH_V1` for 32,768
synthetic nodes and requires zero optimized vector-growth events versus a positive legacy count,
which is a 100% growth-event reduction.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, focused tests, ignored performance evidence, manifest-only commit/push, and one-shot
WeCom publication after the declared gates pass.

## Current batched validation handoff (2026-08-30)

Runtime501 is batched with Editor501 under request
`runtime501-animation-graph-editor501-selection-root-capacity-20260830cn-v1`. Receipt, ticket,
source manifest, and terminal evidence are recorded after coordinator acceptance.
