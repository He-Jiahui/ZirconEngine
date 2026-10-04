---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-28-asset-pointer-item-generation-ownership.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
related_code:
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
tests:
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
---

# Asset Generation Replacement Group Capacity

`AssetWorkspaceItemGeneration::replace_existing_items` now consumes the replacement iterator
through its `size_hint` lower bound and reserves a chunk-grouping `HashMap` capped by the current
chunk count. This removes avoidable outer-table growth for exact-size replacement batches while
remaining conservative for iterators that do not provide an upper bound. Replacement validation,
chunk-local cloning, selected-index updates, and identity sharing are unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor01 / item-generation replacement | Bound replacement chunk-group capacity from the iterator lower bound | implemented_pending_validation | Rust source regression asserts the `size_hint` lower-bound reserve and chunk-count cap. Scoped Rustfmt and diff checks pass. Managed Editor Cargo and release p50/p95/p99 allocation evidence remain pending; no coordinator state was polled in this slice. |

## Source snapshot

| File | SHA-256 |
|---|---|
| `zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs` | `1EC1AF89E23A161CCB561F86DBB1D34AC81591632B83567BC6FBA62329ACC48B` |
