---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-hit-grid-reverse-map-reuse.md
related_records:
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
  - docs/plans/astra/features/runtime/729-runtime-hit-grid-cell-iterator.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
tests:
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - tools/tests/test_runtime_ui_hit_grid_reverse_map_reuse_contract.py
---

# Runtime hit-grid reverse-map capacity reuse

`UiHitTestIndex::reindex_entry_cells` rebuilt the reverse `node_id -> cell list`
map by clearing the map and allocating a fresh `Vec<usize>` for every entry on
each rebuild. The hot rebuild path now retains entries whose node IDs remain in
the new grid, clears only their cell contents, and inserts empty vectors only
for newly observed IDs. Removed IDs are still discarded, and cell membership
ordering and lookup semantics are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A/11C hit-grid reverse index | Reuse stable entry map nodes and per-entry cell-vector capacity across grid rebuilds. | RED/GREEN source guard, capacity-preservation regression, focused Runtime/Editor batch, and scoped Rustfmt pass; managed Cargo/Release allocation and latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

The rebuild still visits each grid cell membership once. For stable entry IDs,
`entry_cells` map nodes and their `Vec` buffers now survive a rebuild; the
steady-state path performs `clear` plus membership pushes without re-allocating
those buffers (unless a new membership count exceeds the retained capacity).
New IDs allocate normally and removed IDs are dropped. The independent
`entry_indices` authority and all fail-closed transactional restoration paths
remain unchanged.

## Local evidence

- The source-contract RED run rejected the old `entry_cells.clear()` and
  `(entry.node_id, Vec::new())` construction; the GREEN run requires retained
  entries, `cells.clear()`, and insertion only through `or_default()`.
- The lower Rust regression builds a wide entry, records its reverse-cell
  capacity, rebuilds the same node as a small entry, and verifies that the
  one-cell result retains the prior capacity.
- Current source SHA-256: `B99A1D7BF598A6378924E33A7DA55BB94A78B32770F128910233EEBFC3A5773B`
  (`hit_test.rs`) and `A69696B285A2E80C327BC54894354998285A76285DF1A608DBEF382030F4D9C8`
  (`geometry_patch.rs`).
- Focused reverse-map and hierarchy-projection contracts pass `4/4`. The
  single-process non-tooling Runtime/Editor performance-plus-pressure batch
  covered 351 modules and passed `1345/1345` tests in `39.080s`.
- A subsequent same-source rerun of that shared non-tooling batch covered 351
  modules and passed `1346/1346` tests in `8.890s`.
- The adjacent Hit-Test/Hierarchy contract batch (25 modules) also passed
  `129/129` tests in `21.376s`.
- The wider Runtime UI plus Editor projection/pointer/cache batch passed
  `760/760` tests across 164 modules in `20.998s`.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

The existing owner-attributed Windows Release admission remains deferred by the
external dirty `E:\Git\zr_vm` worktree gate. No new coordinator request or
status query was issued for this slice. Include this record in the next batched
Runtime/Editor validation; keep the status at `implemented_pending_validation`
until reverse-map allocation and hit-test latency gates are measured.
