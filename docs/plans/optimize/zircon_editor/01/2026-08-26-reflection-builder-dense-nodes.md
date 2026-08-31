---
title: Editor01 Reflection Builder Dense Nodes
category: zircon_editor
report_id: Editor01-reflection-builder-dense-nodes-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Reflection Builder Dense Nodes

## Scope

This slice replaces `SnapshotBuilder`'s construction-time `BTreeMap` with a dense node vector.
Builder-owned `UiNodeId` values start at one and increase by one, so parent mutation now resolves
the vector slot directly instead of performing an ordered-map lookup.

The public `UiReflectionSnapshot` contract remains unchanged. `finish` projects the already
ID-ordered vector into the required `BTreeMap` once, preserving deterministic serialization,
root identity, authored child order, and all node descriptors.

## Performance Workload

The release workload builds a 16,384-node chain, mutates each previously inserted parent, and
materializes the same final ordered map.

| Work per snapshot build | Before | After |
|---|---:|---:|
| Ordered node insertions | 16,384 | 0 |
| Ordered parent lookups | 16,383 | 0 |
| Direct vector parent mutations | 0 | 16,383 |
| Ordered output projections | implicit per insertion | 1 at finish |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_REFLECTION_BUILDER_DENSE_NODES_BENCH_V1`. Acceptance requires dense construction P95 to
be at least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826cg_reflection_builder_dense_nodes_preserves_snapshot_order` covers
  node IDs, root identity, child order, descriptor retention, and ordered snapshot keys.
- `optimization_batch_20260826cg_reflection_builder_dense_nodes_projects_order_once` locks the
  dense construction owner, direct parent lookup, and single finish projection.
- `optimization_batch_20260826cg_reflection_builder_dense_nodes_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained-tree invalidation, partial reflection updates, accessibility
projection, multi-window lifecycle, and product-scale UI qualification. This slice only converges
the reflection snapshot's private construction-time node storage.
