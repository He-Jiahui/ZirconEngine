---
title: Editor23 Hierarchy Streaming Traversal
category: zircon_editor
report_id: Editor23-hierarchy-streaming-traversal-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Hierarchy Streaming Traversal

## Scope

This slice removes repeated document-wide node lookup and full node-ID vector cloning from UI asset
hierarchy presentation and selection. Root-document traversal still excludes component definitions;
component-only documents still follow component map order and child depth-first order.

## Change

- Add one callback-based depth-first traversal over direct `UiNodeDefinition` references.
- Build hierarchy labels from borrowed component/type names without looking each node up again.
- Resolve the selected hierarchy row by streaming until the selected node is found.
- Resolve a clicked hierarchy row by streaming to its index, then clone only the one selected ID.
- Remove the private full-vector `hierarchy_node_ids` path.

## Deterministic Performance Evidence

| Representative flat hierarchy with 2,048 nodes, selecting the final row | Before | After |
|---|---:|---:|
| Node visits from repeated `document.node(id)` lookup | 2,098,176 | 2,048 |
| Node-ID string clones for selected-row lookup | 2,048 | 0 |
| Node-ID string clones for click-by-index | 2,048 | 1 |
| Traversal order | depth-first source order | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR23_HIERARCHY_STREAMING_TRAVERSAL_BENCH_V1`. Acceptance requires streaming P95 to be at most
60% of cloned repeated-lookup P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ai_editor23_streaming_hierarchy_preserves_root_and_component_order`
  covers nested root order, ignored component definitions, component-only order, first/last rows,
  and out-of-range lookup.
- `optimization_batch_20260826ai_editor23_hierarchy_uses_direct_streaming_traversal` requires the
  shared direct-node DFS and rejects full-vector clone/`nth` selection.
- `optimization_batch_20260826ai_editor23_hierarchy_streaming_traversal_performance_evidence`
  checks selected-row equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts passed before managed
  validation submission.

## Remaining Parent-plan Work

Editor23 still needs complete preview/authoring product traces, large-document memory qualification,
transactional save/reload coverage, and end-to-end accessibility validation. This slice only
converges hierarchy projection and selection traversal.
