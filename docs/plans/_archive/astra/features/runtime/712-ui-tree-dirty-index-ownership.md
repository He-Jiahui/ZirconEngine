---
title: Runtime11A UiTree Dirty-Index Ownership and Bulk Mutation Tracking
category: zircon_runtime
report_id: Runtime11A-ui-tree-dirty-index-ownership-2026-09-12
date: 2026-09-12
session_id: root-astra-optimize-20260909
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/zircon_runtime/runtime/09/2026-08-09-ui-architecture-performance-reassessment.md
implementation_files:
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - zircon_runtime/src/ui/surface/invalidation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/rebuild/authored_geometry.rs
tests:
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - zircon_runtime/src/ui/surface/invalidation/domain_bitset_tests.rs
  - tools/tests/test_runtime_incremental_layout_snapshot_performance_contract.py
  - tools/tests/test_runtime_ui_surface_incremental_rebuild_owner_structure.py
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A UiTree Dirty-Index Ownership and Bulk Mutation Tracking

## Scope

This slice makes the retained `UiTreeNodes` dirty-domain index authoritative for mutable tree
access, removes temporary full key-set snapshots from bulk mutation tracking and surface dirty
candidate assembly, and keeps the authored-geometry subset check on the journal itself. The
existing incremental surface summary, effective state-flag projection, removal handling, and
serialized payload shape remain unchanged.

## Implementation

- `UiTree::node_mut` retains the `UiTreeNodes::get_mut` ownership boundary, while that mutable
  entry point now enqueues the node for dirty-index reconciliation and invalidates the paint-order
  cursor just like other mutable access paths.
- `UiTreeDirtyIndex` keeps per-node effective flags and seven domain counters, performs one full
  build for a deserialized/initial tree, and then refreshes only pending node IDs.
- Clearing mutation bookkeeping defers that first full build when the index is still cold; an
  already initialized index is reconciled before pending IDs are discarded.
- `track_all_nodes` now inserts keys directly into both pending sets while iterating the retained
  map; it no longer allocates an intermediate `BTreeSet` containing every key.
- Surface dirty-summary, pending-rebuild, and clear paths extend their caller-owned candidate
  sets directly from the invalidation journal and dirty index. Authored-geometry validation uses a
  streaming subset predicate instead of materializing a second pending-ID set.
- The dirty summary now appends indexed `(node_id, flags)` pairs directly into its owned output
  buffer, avoiding the intermediate `dirty_node_entries()` vector; the incremental candidate
  summary reserves its bounded destination once.
- The regression verifies that direct `node_mut` changes are observed, idle queries do not rescan
  nodes, cold bookkeeping clears do not force a scan, deserialized trees perform one full index
  build before pending-node refreshes,
  `values_mut` marks all nodes without an intermediate key snapshot, and the direct-set extension
  helpers preserve membership/subset semantics.

## Complexity and performance boundary

| Path | Previous shape | Current shape |
|---|---|---|
| Idle dirty-domain query | repeated full node fold | O(1) after pending reconciliation |
| Cold bookkeeping clear | eager full index fold | no scan; first query performs one O(N) build |
| One mutable node update | caller could bypass index | one pending-node refresh |
| `values_mut`/`iter_mut` tracking | map scan plus temporary key set | one map scan, direct inserts |
| Surface dirty candidate merge | several owned temporary key sets | direct extension into one caller-owned set |
| Dirty summary entry projection | index vector plus summary copy | direct append into summary buffer |
| Authored-geometry pending subset | materialize journal keys then compare | journal-key membership predicate |
| Deserialized first query | no reliable side-index ownership | one explicit O(N) index build |

The retained `dirty_node_ids()` and `dirty_node_entries()` APIs still materialize their requested
owned outputs; this slice does not claim those caller-visible copies are free.

## Local verification

- Focused tree source regression covers direct mutation, idle no-rescan behavior, and bulk
  tracking; the source guard rejects `collect::<BTreeSet<_>>()` in `track_all_nodes`.
- Batched Runtime/Editor performance-contract discovery: `1340/1340` tests passed in `6.875s`.
- Current isolated Runtime/Editor performance-contract discoveries also pass:
  Runtime `1146/1146` in `6.603s` and Editor `581/581` in `0.890s`; the current
  pressure batches pass Runtime `142/142` in `3.145s` and Editor `122/122` in
  `3.255s`.
- One combined Runtime/Editor batch loaded all 535 matching performance-contract and
  pressure modules in a single invocation: `1991/1991` tests passed in `8.659s`.
- After the Runtime712 dirty-summary extension and parallel Editor713 cache-remap changes, the
  refreshed combined batch loaded 536 modules and passed `1993/1993` in `12.193s`; the Runtime
  dirty-index source and focused regressions remain included in that current-source sweep.
- Runtime714's text-decoration DTO fallback was applied afterward; a fresh one-invocation sweep
  still loaded 536 modules and passed `1993/1993` in `12.015s`.
- The latest combined Runtime layout/surface and Editor cache subset loaded sixteen contracts and
  passed `106/106` in `1.213s`.
- A broader non-tooling Runtime/Editor discovery ran `3468` tests and retained eight
  unrelated boundary/naming/tech-stack failures; those owners are outside this slice and
  were not changed.
- A focused Runtime/Editor UI batch covering 48 dirty/layout/surface/tree/hit/invalidation
  modules passed `240/240` in `28.741s`.
- A broader non-tooling Runtime/Editor discovery (3,468 tests) still reports eight
  unrelated pre-existing boundary failures in preview-token parity, WOC preference
  dependencies, runtime/editor naming classification, render legacy naming, and tech-stack
  anchor checks; those owners are outside this slice and were not changed.
- Post-extension focused Runtime layout/surface contract batch: `65/65` tests
  passed in `0.641s`, including incremental layout, dirty-rebuild ownership,
  layout-order, resize, frame-domain, and hit-query contracts.
- Rustfmt parser invocation and scoped `git diff --check` passed for the affected Runtime tree and
  surface files.
- Managed Windows Cargo behavior and Release allocation/time p50/p95/p99 evidence remain pending;
  no product-performance acceptance is inferred from local source contracts.

Tooling remains intentionally outside this slice. No coordinator status was polled and no new
coordinator request was issued.
