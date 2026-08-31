---
title: Runtime94 Indexed Virtual Geometry Hierarchy Nodes
category: zircon_runtime
report_id: Runtime94-indexed-virtual-geometry-hierarchy-nodes-2026-08-26
date: 2026-08-26
session_id: root-runtime94-virtual-geometry-hierarchy-index-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime94 Indexed Virtual Geometry Hierarchy Nodes

## Scope

This slice optimizes hierarchy-node resolution while building the virtual-geometry node and cluster
cull debug snapshot. It removes a full `hierarchy_nodes` scan from every traversal queue visit. It
does not change authored hierarchy order, breadth-first traversal order, child expansion, cluster
storage records, page requests, GPU word layouts, public snapshot types, or culling policy.

## Change

- The traversal builder creates one capacity-sized `HashMap<u32, &RenderVirtualGeometryHierarchyNode>`
  before processing its queue.
- Queue visits borrow the indexed node directly instead of scanning the complete authored node
  vector for each visit.
- Index construction uses `entry(...).or_insert(...)`, preserving the previous `iter().find(...)`
  behavior when malformed input contains duplicate node IDs: the first authored node remains the
  resolved node.
- A Rust regression locks the duplicate-ID first-match behavior, while a Python source contract
  prevents the per-visit linear scan from returning.

## Deterministic Performance Evidence

The independent release model uses 2,048 hierarchy nodes and 16,384 reverse-order node lookups,
with 21 alternating legacy/indexed sample pairs per run. Index construction is included in every
indexed timing sample.

| Evidence | Linear lookup | Borrowed index | Result |
|---|---:|---:|---:|
| Comparisons/index operations | 16,785,408 | 18,432 | 99.890% fewer |
| Run 1 P50 | 9.215 ms | 0.422 ms | 95.43% faster |
| Run 1 P95 | 14.256 ms | 0.570 ms | 96.00% faster |
| Run 2 P50 | 10.111 ms | 0.477 ms | 95.28% faster |
| Run 2 P95 | 13.487 ms | 1.014 ms | 92.48% faster |
| Run 3 P50 | 10.250 ms | 0.468 ms | 95.43% faster |
| Run 3 P95 | 20.593 ms | 2.229 ms | 89.18% faster |

The managed gate requires the exact deterministic operation counts above, at least 99% operation
reduction, at least 80% P50 improvement, and at least 50% P95 improvement.

## Acceptance

- `tools.tests.test_runtime94_virtual_geometry_hierarchy_index_performance_contract` passes 3/3
  locally.
- Exact-file `rustfmt --check` and scoped `git diff --check` pass locally.
- The `runtime94_hierarchy_node_index_preserves_first_authored_duplicate` regression, source
  contracts, formatting, performance model, and relevance-index slice are submitted together in
  one coordinator-managed two-task validation batch.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime94 still needs production GPU-driven visibility evidence, spatial-index and HZB budget
validation, large-scene residency pressure measurements, batching/instancing acceptance, and
Editor/game product-path profiling.
