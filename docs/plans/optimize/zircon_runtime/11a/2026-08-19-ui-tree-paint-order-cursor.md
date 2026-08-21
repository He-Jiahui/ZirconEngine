# Runtime11A UI Tree Paint-Order Cursor Optimization Record

- Date: 2026-08-19
- Owner: `runtime11a-paint-order-cursor-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md`, P1-12
- Status: implementation and 21-pair measurement repair complete; combined managed validation pending

## Problem

`UiTree::insert_root` and `UiTree::insert_child` scanned every existing node to find the maximum `paint_order`. Building an N-node tree therefore performed `N * (N - 1) / 2` node visits before the map insertion cost, making template construction O(N^2).

## Change

- `UiTreeNodes` now owns a private monotonic paint-order cursor.
- New trees allocate paint order in O(1) without scanning existing nodes.
- Deserialized trees keep the serialized map shape unchanged and rebuild the skipped cursor once on first insertion.
- Public mutable node entry points invalidate the cursor because callers can edit `paint_order`; the next insertion rebuilds once while retaining the monotonic allocation high water.
- Removing the highest node never makes its retired paint order reusable, even when an earlier mutable borrow forced a rebuild.
- Internal parent-child insertion mutates the parent through a cursor-preserving path because it cannot change paint order.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Insert 10,000 roots into a new tree | 49,995,000 existing-node visits | 0 existing-node visits | 100% |
| Insert one root and 10,000 children | 50,005,000 existing-node visits | 0 existing-node visits | 100% |
| Insert twice after deserializing 4,096 nodes | 8,193 existing-node visits | 4,096 existing-node visits | 50.0% |
| Asymptotic bulk construction | O(N^2) paint-order lookup | O(N) total construction plus map insertion | one full complexity class |

The figures count deterministic paint-order scan visits, excluding the unchanged `BTreeMap` insertion cost.

## Acceptance

- `bulk_insert_assigns_dense_paint_order_without_rescanning_existing_nodes`
- `bulk_child_insert_preserves_the_cursor_while_mutating_the_parent`
- `deserialized_tree_rebuilds_paint_order_cursor_only_once`
- `mutable_node_access_invalidates_the_paint_order_cursor`
- `cursor_rebuild_does_not_reuse_a_retired_high_water_order`
- `paint_order_cursor_release_benchmark_evidence` emits 21 paired, alternating
  legacy/cursor samples and requires nearest-rank cursor P95 to be no more than
  25% of legacy P95.
- `rustfmt +1.94.1 --edition 2021 --check zircon_runtime_interface/src/ui/tree/node/ui_tree.rs`: passed
- `git diff --check -- zircon_runtime_interface/src/ui/tree/node/ui_tree.rs`: passed
- Cargo compile/test and release measurement: pending the next multi-task coordinator batch; no per-task Cargo run was started.
