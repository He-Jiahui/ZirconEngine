---
title: Editor01 Extension Menu Operation Index
category: zircon_editor
report_id: Editor01-extension-menu-operation-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Extension Menu Operation Index

## Scope

This slice removes a recursive full-menu scan for every contributed editor view. Existing menu
item ordering, extension priority ordering, nested branches, command enablement, view iteration,
and first-view insertion behavior remain unchanged.

## Change

- Traverse the completed base/extension menu tree once and collect cloned `EditorOperationPath`
  keys into a `HashSet`.
- Use `insert(operation_path.clone())` to combine duplicate detection with index maintenance.
- Add each newly published view operation to the same index before appending its menu item, so
  later duplicate view contributions remain suppressed.
- Remove the per-view recursive `item_contains_operation` scan.

## Deterministic Performance Evidence

| 4,096 existing menu operations and 2,048 reverse-order view probes | Before | After |
|---|---:|---:|
| Menu-item visits | 6,292,480 | 4,096 index inserts + 2,048 average `O(1)` probes |
| Nested operation coverage | recursive scan | one recursive index build |
| View publication order | contribution order | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_EXTENSION_MENU_OPERATION_INDEX_BENCH_V1`. Acceptance requires indexed P95 to be at most
60% of recursive-scan P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ak_menu_operation_index_covers_nested_items` verifies nested,
  duplicate, new, and borrowed-string lookup behavior.
- `optimization_batch_20260826ak_extension_views_use_one_operation_index` requires the shared
  operation index and rejects the recursive per-view scan.
- `optimization_batch_20260826ak_extension_menu_operation_index_p95` reports both P95 values and
  enforces the 60% threshold.

## Remaining Parent-plan Work

Editor01 still needs full retained shell, menu invalidation, contribution churn, large-extension,
and product P95 qualification. This slice only converges extension-view menu deduplication.
