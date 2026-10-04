---
title: Editor01 Asset Browser thumbnail-node bounded capacity
category: zircon_editor
report_id: Editor790-thumbnail-node-capacity-2026-09-17
date: 2026-09-17
session_id: root-runtime-editor-async-optimization-20260917
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor790 · Asset Browser thumbnail-node bounded capacity

## Scope

`append_asset_browser_thumbnail_slots` appends one grid panel and exactly nine
template nodes for each materialized thumbnail item. The append path previously
started from the caller's current vector capacity, so a cold thumbnail
projection could grow geometrically even though both the visible asset count
and the materialization window were already known. This slice is limited to
that bounded allocation shape; virtualization, ordering, and thumbnail paint
semantics remain unchanged.

## Implementation

- Added `thumbnail_node_capacity`, using
  `1 + min(visible_item_count, materialized_item_count) * 9` with saturating
  arithmetic as an additional-capacity upper bound.
- Reserved that bound immediately after the thumbnail-mode guard and before
  appending the grid panel or item nodes.
- Kept the existing `take(materialized_item_count)` traversal, selection
  resolution, node order, control IDs, and non-thumbnail early return intact.

## Deterministic pressure model

The 4,096-item model emits 36,865 nodes (one panel plus nine nodes per item).
Starting from a zero-capacity vector, the legacy geometric model reports 15
growth events; the bounded reservation reports 0. The lower ignored marker
`EDITOR790_THUMBNAIL_NODE_CAPACITY_BENCH_V1` carries the same comparison. This
is allocation-shape evidence, not CPU, RSS, allocator, or product p50/p95/p99
evidence.

## TDD and local evidence

The Python source contract was intentionally RED while the old append path had
no capacity helper, then GREEN at `3/3` after the reservation and test-module
wiring were added. The lower module checks the exact bound, the visible/requested
minimum, and overflow saturation, and carries the ignored Release marker.
Scoped Rustfmt and the merged Runtime/Editor source-contract and regression
batches pass `557/1990` performance-contract checks in `71.984s` and
`921/3740` broad non-tooling regression checks in `882.184s`, with zero
failures, errors, or skips. These local source/model receipts are recorded in
the linked Astra completion ledgers.

## Source files

- `zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs`
- `zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes/capacity_tests.rs`
- `tools/tests/test_editor_thumbnail_node_capacity_performance_contract.py`

## Managed acceptance gate

Keep this slice `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Editor tree,
runs the lower regression and ignored marker, and supplies allocator plus
Asset Browser thumbnail projection p50/p95/p99 evidence. No standalone Cargo
command or coordinator status query is used here; tooling production work
remains deferred for the later Rust migration.
