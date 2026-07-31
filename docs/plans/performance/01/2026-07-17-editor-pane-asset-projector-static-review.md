# Editor Pane And Asset Projector Static Review

- Date: 2026-07-17
- Scope: `paint_workbench_renderer/{docks/pane.rs,docks/pane/**}`
- Rust files read: 10/10
- Lines read: 919
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-219`
- Fixing plan: `docs/plans/zircon_editor/editor/09-editor-asset-management.md`

## Files reviewed

| module | files | result |
|---|---:|---|
| pane shell/content/fallback | 4 | visible-frame guard and constant viewport/template/native/debug/fallback dispatch; no unbounded local loop |
| template selection | 2 | bounded pane-kind routing; transformed paths share the PERF-MVP-218 owned-row clone fix |
| asset identity/projector | 3 | repeated initialization scans and identity parsing before the main command traversal |
| tests | 1 | list/thumbnail, scroll, clip, hover, empty, stale, header/grid, and preview behavior covered; no scan/clone/runtime scale counter before this change |

## Bottleneck evidence

`ActivityAssetContentProjector::new` independently traversed `ModelRc` to find the content panel and count folder rows, cloning every visited DTO through `row_data` twice. Browser mode first searched for a thumbnail grid and, on list mode, restarted from row zero for table, header, and preview. Its worst-case initialization was four full scans. The command pipeline then traversed the model again to transform and paint nodes.

## Direct fix

Activity now gathers the first panel frame and folder-row count in one pass. Browser gathers grid/table/header/preview geometry in one pass and exits as soon as a thumbnail grid fixes the mode. Only compact frame scalars survive initialization; no full DTO is retained. A source guard requires exactly one loop per constructor and prohibits the old search helpers. The existing behavioral suite remains the parity authority until current-source Cargo runs.

The adjacent native Browser content scrollbar used the same four independent lookups. It now collects grid or list table/header/preview geometry and extent in one pass, with a focused source guard requiring one production loop and no `find_node` helper. Activity content scrollbar was already one-pass; asset tree hover/count remain separate dynamic consumers pending the generation index.

## Reference-engine direction

- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h` owns an item source and generated widgets for visible rows rather than rediscovering structural controls during paint.
- Godot `dev/godot/scene/gui/item_list.cpp` stores each item's `rect_cache` and marks shape dirty on mutations, separating geometry generation from stable drawing.

Editor09 should publish the asset content mode, fixed geometry, parsed identities, row counts, and visible range with the asset-model generation. Scroll and hover then patch only dynamic fields, and stable paint performs no projector-wide model scan.

## Dynamic acceptance still required

- Run current-source projector behavior tests and `zircon_editor --lib performance_tests`.
- Measure 1, 1,000, and 10,000 nodes for `row_data`, identity parses, DTO clone bytes, allocations, and CPU p50/p95/p99.
- Prove stable generation projector initialization scans are zero after the Editor09 handoff lands.
- Preserve Activity/Browser list and thumbnail layout, header/grid/preview boundaries, scroll, clip, hover, empty/stale state, hit behavior, and Softbuffer pixels before moving the folder to `review.md`.
