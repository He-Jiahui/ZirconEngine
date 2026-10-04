---
title: Editor retained hierarchy typed row receipt hard cutover performance review
date: 2026-08-23
module: zircon_editor retained-host hierarchy_pointer
priority: MVP-P0 hierarchy selection scroll hover and scene drag
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine SSceneOutliner and SListView
---

# Goal

Keep the native hierarchy pane as the sole pane hit authority, preserve the current O(1) arithmetic
row lookup and visible-row-only paint, and remove the editor-side generic hit surface plus the O(N)
row clone/string identity projection performed for pointer routing. A hierarchy action must resolve
an item index against the exact committed typed row allocation instead of formatting, cloning and
parsing entity identifiers.

## Reviewed source

- owner Rust files: 22/22
- physical lines: 628
- bytes: 21,643
- LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `6d24dafe43cc5be6cb17c4bbefdd14685e521e865189bd8a9c9fb01cc757e35d`
- owning commit at review: `08094b9b9e17f6c80372e15c17b01204038b305b`
- post-M0 owner Rust files: 15/15
- post-M0 physical lines: 360
- post-M0 bytes: 11,863
- post-M0 LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `88d47383933a9732f4bf434bf4438d9a3c0310bfacb7a9884f1688c5884dff59`

All owner files were read in full. The review also traced native pane button/move/scroll routing,
callback storage and wiring, committed hierarchy refresh/filter projection, pointer target
preparation, selection dispatch, exact-row rename lookup, drag-source/reparent resolution,
automation coordinates, retained-list tests, native visible-row paint and prior hierarchy reports.

The 2026-07-17 report's claim that scrolling rebuilds every logical hierarchy row is stale and was
retested against current source. `rebuild_surface` now creates only root and viewport nodes,
`handle_scroll` does not rebuild, `route_at_point` computes the row with `floor(content_y / pitch)`,
and native paint calls `visible_hierarchy_row_range`. Current row hit cost is O(1), retained hit-node
count is 2, and paint work is O(V) for visible rows. Those algorithms must be preserved.

## Structural findings

### P0: committed typed rows are copied and converted to strings for a pointer-only mirror

`SceneEntries` already retains hierarchy rows as `Arc<[WorldInspectionHierarchyRow]>`, but
`sync_hierarchy_pointer_layout` accepts a slice and performs `Arc::from(scene_entries)`, cloning all
rows into another allocation. It then formats every typed `entity` into a new `Vec<String>` for
`HierarchyPointerLayout.node_ids`. Thus a pointer projection on an N-row reflow performs O(N) row
clones, O(N) formatting and N string allocations after the workbench projection already exists.

The bridge only needs pane size and item count for arithmetic hit testing. M0 must pass the existing
row `Arc` by reference count, retain it as the click/drag authority, and reduce bridge equality to
three scalar fields. Pointer projection then becomes O(1) with no per-row allocation.

### P0: one native pane hit is sent through a second generic hit tree

Native `PanePointerRoute::Hierarchy` has already proved the pointer belongs to the committed
hierarchy pane and forwards local coordinates and size. `HierarchyPointerBridge` then dispatches a
second event through a two-node `UiSurface`, `UiPointerDispatcher` and `EditorRouteIntentMap` only to
recover `ListSurface`; `project_route_at_point` subsequently performs the real arithmetic row hit.

The two-node surface does not own row identity or paint geometry and adds a second authority that
can fail independently. M0 must validate the local point directly against the pane viewport and
return `Node { item_index }`, `ListSurface`, or `None` in one arithmetic route.

### P0: click identity is degraded from typed entity to String and parsed back

`route_at_point` clones the selected `node_id: String`; shared click dispatch parses it back into
`NodeId`, while rename and drag paths independently use the same `item_index` to recover the typed
row. The source row already contains a Copy `NodeId`. M0 must make routes Copy/index-only and resolve
the typed row once in shared click dispatch, returning that selected entity for rename reuse.

### P1: generic dispatch errors and rebuild counters survive after their authority is obsolete

`handle_click`, `handle_move` and `handle_scroll` return `Result` only because the generic surface
dispatch can fail. Geometry changes patch and rebuild the two-node surface and increment four
hierarchy surface/dispatcher/route counters. Direct arithmetic routing removes that fallible path;
runtime command dispatch remains fallible at the actual selection/reparent boundary.

## Zircon and Unreal source basis

Direct Zircon source read:

- `ui/workbench/snapshot/data/scene_entry/entries.rs` owns typed immutable rows in an `Arc` and
  exposes `hierarchy_rows_arc`; another row allocation is not an authority boundary.
- `host_contract/native_pointer/**/hierarchy.rs` proves the native pane target before invoking the
  hierarchy callback.
- `hierarchy_pointer/route_at_point.rs` already contains the correct O(1) pitch/index algorithm.
- `host_contract/paint_workbench_renderer/native_panes/hierarchy.rs` bounds row drawing to
  `visible_hierarchy_row_range`, so current paint complexity is O(V), not O(N).

Direct Unreal source read:

- `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp` binds
  `TreeItemsSource` to retained `FSceneOutlinerTreeItemPtr` items and passes the exact typed item to
  `OnOutlinerTreeSingleClick`, `OnItemClicked` and `SetItemSelection`.
- `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/SOutlinerTreeView.h` declares rows
  over `FSceneOutlinerTreeItemPtr`; row widgets retain the matching typed item.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h` starts generation
  at the scroll-derived index, stops after filling the available view, reuses a widget already
  mapped to the item and cleans up widgets not seen during the generation pass.

The transferable rule is retained typed item identity plus view-bounded row generation. A pointer
subsystem should not create an all-row string identity mirror or a second generic pane hit tree.

## Target architecture

1. `SceneEntries::hierarchy_rows_arc()` supplies the committed typed row allocation to pointer state
   with an O(1) `Arc` clone.
2. `HierarchyPointerLayout` stores `pane_width`, `pane_height` and `item_count` only.
3. One direct arithmetic route returns a Copy item-index receipt; no `UiSurface`, dispatcher or
   route-intent binding exists in this owner.
4. Shared click dispatch validates the index against the same row allocation once, dispatches the
   typed `NodeId`, and exposes it for exact-row rename lookup.
5. Native paint remains O(V), pointer hit remains O(1), stable pointer sync becomes O(1), and O(N)
   work is permitted only when the authoritative hierarchy/filter projection actually changes.
6. A later generation receipt must bind native paint, pointer item count and typed rows so stale
   callbacks cannot resolve against a newer same-length projection.

## Instrumentation and acceptance

Matrix: rows `0/1/100/10K/100K`; viewport rows `4/16/64`; operation
`stable-sync/reflow/move/click/scroll/drag-start/drop`; topology `stable/add/remove/reorder/filter`;
input rate `10/125/500 Hz`; receipt generation `current/stale`.

Acceptance requires:

- pointer-projection row clones per reflow: `N -> 0` at M0;
- pointer-projection entity formatting/string allocations per reflow: `N -> 0` at M0;
- pointer layout equality work: O(N) string comparison -> O(1) scalar comparison at M0;
- generic mirror hit dispatches per native hierarchy callback: `1 -> 0` at M0;
- selected route-owned String payloads and NodeId parses: `1 -> 0` at M0;
- native hierarchy paint remains bounded by visible rows plus constant overscan;
- current-generation selection, hover, scrolling, rename, drag source and root/node drop remain
  behaviorally equivalent;
- stale generation/index receipts are rejected deterministically at M1;
- p95 pointer routing below 0.01 ms at 100K rows and no row-count-correlated allocation after M0;
- WPR/allocator evidence shows no mirror hit-dispatch or all-row pointer projection stack.

RenderDoc is relevant only to final hierarchy pixel/draw parity because M0 changes neither row
geometry nor renderer commands. WPR, allocator, executable and capture artifacts must remain on
D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Reuse typed row Arc; item-count layout; direct arithmetic hit; index-only route; delete generic mirror surface and String round trip. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | Publish one generation across hierarchy paint rows, item count and pointer receipts; reject stale callbacks. | reorder/filter race and generation tests |
| M2 | Audit hierarchy projection/selection transactions and invalidation so stable input publishes only interaction damage. | exact invalidation and no unrelated snapshot/reflow |
| M3 | Run scale/storm/WPR/allocator/power plus interaction and RenderDoc pixel/draw parity. | quantified acceptance and closeout |

## Validation state

- Owner review: complete, 22/22 Rust files.
- Native callback, committed projection, selection/rename/drag/automation, tests and visible paint:
  read and mapped.
- Unreal Scene Outliner typed-item and Slate view-bounded row-generation source: read and mapped.
- Architecture report: recorded before implementation.
- M0 implementation: applied. `HierarchyPointerLayout` now contains only pane size and item count;
  the app retains `SceneEntries::hierarchy_rows_arc()` instead of cloning rows and formatting entity
  strings. Click/move/scroll call the direct arithmetic route, selection resolves one typed row,
  and seven mirror surface/dispatcher/route-map support files plus the hierarchy route-intent
  variant are deleted.
- Exact static owner delta: files `22 -> 15`, physical lines `628 -> 360` (-268, 42.7%), bytes
  `21,643 -> 11,863` (-9,780, 45.2%). Pointer-projection row clones and entity string allocations
  are `N -> 0`; generic mirror dispatches per callback are `1 -> 0`; route String clones and NodeId
  parses are `1 -> 0`. These are source/operation-count facts, not timing claims.
- Focused static contract:
  `tools/tests/test_editor_retained_hierarchy_typed_row_receipt_performance_contract.py`, 131 lines,
  4,696 bytes, SHA256
  `c19e526e90d7b89b5b177076ef5f62af393bd77aa7656e40facc0ee1548ed059`; RED 0/9 to GREEN 9/9.
- Retained-host Python performance contracts: GREEN 49/49. Profile-capture Pester contract: GREEN
  45/45 using `E:\ZirconTemp\pester-hierarchy-m0`. Rustfmt and scoped `git diff --check` passed.
- Broad current-worktree performance discovery ran 222 tests: 220 passed and two errored in the
  separate untracked asset-browser preview-materialization contract because its active worktree
  slice references a missing `preview_artifact.rs` and an old paint helper. Those errors do not
  touch hierarchy paths and are not counted as hierarchy acceptance.
- M1-M3 and dynamic evidence remain pending; this owner stays in `pending.md`.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is archived and returns
  `cargo_session_not_executable`; raw Cargo is not an allowed bypass.
