---
title: Runtime77 Tree Reparent Borrowed Traversal
category: zircon_runtime
report_id: Runtime77-tree-reparent-borrowed-traversal-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Tree Reparent Borrowed Traversal

## Scope

This slice removes clone amplification and temporary ID projections from the runtime tree-view
reparent path. Source/parent validation, first-match behavior, depth-first flattened indices,
child-property priority, and reparented output order remain unchanged.

## Change

- Carry the removed `UiValue` in one `Option` and consume it only when the destination parent is
  reached; recursive candidate traversal no longer clones the entire source subtree.
- Return borrowed `&str` identities from node maps instead of cloning each ID during traversal.
- Compute source positions and descendant membership through early-exit borrowed DFS rather than
  building temporary `Vec<String>` projections.

## Deterministic Performance Evidence

| 2,048 candidate parents, 65-node source subtree | Before | After |
|---|---:|---:|
| Source-subtree clones before final parent | 2,047 | 0 |
| Position temporary ID strings | one full vector per lookup | 0 |
| Descendant temporary ID strings | up to one full source subtree | 0 |
| Traversal/order semantics | depth-first, property priority | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME77_TREE_REPARENT_BORROWED_TRAVERSAL_BENCH_V1`. Acceptance requires borrowed insertion P95
to be at least 80% below clone-per-candidate P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826am_tree_reparent_preserves_order_and_rejects_cycles` covers
  flattened from/to indices, nested child placement, and cycle rejection.
- `optimization_batch_20260826am_tree_reparent_uses_borrowed_single_owner_traversal` requires the
  single-owner insertion and borrowed position/identity helpers while rejecting the old clone and
  flattened-ID projection boundaries.
- `optimization_batch_20260826am_tree_reparent_borrowed_traversal_p95` reports paired P50/P95
  samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Runtime77 still owns full input dispatch, focus, navigation, pointer capture, gestures, drag/drop,
IME, and window-lifecycle product integration. This slice only converges tree-view reparent data
movement.
