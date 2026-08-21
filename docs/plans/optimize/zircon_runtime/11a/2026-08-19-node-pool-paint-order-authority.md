# Runtime11A Node-Pool Paint-Order Authority Optimization Record

- Date: 2026-08-19
- Owner: `runtime11a-paint-order-cursor-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md`, P1-12
- Status: implementation and 21-pair measurement repair complete; combined managed validation pending

## Problem

`insert_or_reuse_pooled_child` implemented a second child-insertion path. It scanned every retained node for the maximum `paint_order`, mutated the parent twice, inserted directly into `UiTreeNodes`, and duplicated parent/dirty invariants already owned by `UiTree::insert_child`.

## Change

- Reused and newly created pooled children now enter through `UiTree::insert_child`.
- The node-pool-local O(N) `next_paint_order` scan was deleted.
- The duplicate parent mutation and dirty-marking helper were deleted.
- Removed paint orders remain retired; a recycled node receives the next monotonic order instead of reusing an older order.

## Deterministic Performance Evidence

| Operation | Before | After | Reduction |
|---|---:|---:|---:|
| Reinsert one pooled child with 10,000 retained nodes | 10,000 paint-order scan visits | 0 scan visits | 100% |
| Parent mutable map lookups per reinsert | 2 | 1 | 50% |
| Paint-order authorities in the insertion path | 2 | 1 | 50% |

The remaining ordered-map insertion cost is unchanged. The improvement removes the full-tree pre-scan and one duplicate parent lookup.

## Acceptance

- `surface_node_pool_reinsert_uses_the_ui_tree_paint_order_authority`
- `surface_node_pool_reuses_detached_template_node_and_resets_transient_state` now verifies monotonic `paint_order == 2` after detach/reuse.
- `rustfmt +1.94.1 --edition 2021 --check` on the three touched Rust files: passed
- `git diff --check` on the owned paths: passed
- Cargo compile/test and release measurement: pending the next multi-task coordinator batch; no per-task Cargo run was started.
