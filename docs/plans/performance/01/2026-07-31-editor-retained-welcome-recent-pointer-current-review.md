---
related_code:
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer
  - zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-asset-pointer-full-surface-rebuild.md
tests:
  - bridge inline tests inspected: 1
  - retained list-pointer files/tests inspected: 5/9
  - direct rustfmt check: passed for 24/24 current-source files
  - current-source managed Windows Cargo pending
  - recent-row move/scroll counters and WPR/Tracy trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained welcome-recent pointer current review (2026-07-31)

## Scope

`zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs` and its child directory are **4/4** Rust files, **106** physical lines, with path+raw-content SHA-256 `64760c12b994108fdf7a03f8e977d5bf859c7f866403bd3166b90f82f7abff19`. The called `retained_host/welcome_recent_pointer/**` bridge is also reviewed at current source: **20/20** Rust files, **634** physical lines, **1** inline test, SHA-256 `fd7edd267659277a0f098a602a6bb038c8853780286d8f4e0b9b4c6da2c4a045`. The app files are clean; bridge `constants.rs`, `helper.rs`, and `welcome_recent_pointer_bridge_rebuild_surface.rs` contain external uncommitted changes and were reviewed read-only.

The review also traced `pointer_layout/welcome_recent.rs`, `workbench_snapshot_access::welcome_recent_project_paths`, `EditorRouteIntentMap`, shared welcome click dispatch, and all **5/5** files / **9** tests in `retained_list_pointer`.

## Findings

- Positive boundary: the route map uses two `HashMap` indices, and `sync` / `sync_pane_size` skip `rebuild_surface` when layout and state are unchanged. Invalid pane sizes do not build a surface. These guards remain required.
- Every app move first records committed-layout diagnostics, resolves pane size, clones the small pointer state into `sync_pane_size`, and calls `apply_welcome_recent_pointer_state_to_ui`; after dispatch it overwrites the returned state and calls the same UI projection again. The projection performs three Slint setters, so stable same-row motion reaches six setter calls per event even when no hover identity changed.
- `handle_move` asks the route-intent map for an owned `WelcomeRecentPointerRouteIntent`. An action route clones its project-path `String`, then converts it to a public owned route, although the app move consumer discards the route and only uses state. This confirms the move-path part of PERF-MVP-117 at the current adapter boundary.
- Every click builds `runtime.chrome_snapshot()`, collects all recent project paths into a new `Vec<String>`, and deep-compares/synchronizes the complete pointer layout before one-row hit dispatch. A stable generation therefore pays O(N) row visits and path bytes before the bridge equality guard.
- Every effective scroll offset calls `rebuild_surface()`. For N rows it recreates the root/viewport plus three nodes per row, dispatcher handlers, formatted node paths, route maps, and two owned project-path copies per row, then runs `surface.rebuild()`. `virtualization: None` makes scroll work proportional to the full recent list.
- The current source does not need another task number. These symptoms strengthen PERF-MVP-117: EditorUI01 owns stable typed recent-row identities, visible-range materialization, incremental scroll/hit state, and state-only move output; EditorUI08 supplies the changed project-list generation and must not create a second visible-row cache.

## Reference and target

Godot `dev/godot/scene/gui/item_list.cpp:769-776` updates hover and queues redraw only when the hovered identity changes. Its scroll path updates existing scrollbar values (`:1171-1178`), while paint computes a clip and binary-searches the first visible separator/item (`:1471-1496`) rather than rebuilding input nodes for every row. Zircon should keep typed routes and deterministic order, but use the same change-only hover and visible-range principles.

PERF-MVP-117 acceptance should cover recent rows `1/100/10K`, moves `1/1K/1M @125/500/1000Hz`, scroll deltas and click open/remove. Record chrome/project-list snapshots, row visits, path clone bytes, route clones, surface/node/path/dispatcher builds, hit-grid updates, Slint setters, invalidation/redraw, queue age, and UI p50/p95. Stable same-row move must have path clone=0 and UI setters=0; stable click must not build a full chrome/path layout; scroll full-surface rebuild=0 and active nodes must be bounded by viewport+overscan. Open/remove route identity, hover, clamp, order, invalid/missing rows, Cargo, F0/F4, WPR/Tracy, and independent review remain required before moving this module to `review.md`.
