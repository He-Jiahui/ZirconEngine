---
related_code:
  - zircon_editor/src/ui/retained_host/route_intent/mod.rs
  - zircon_editor/src/ui/retained_host/route_intent/map.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_route_intent.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/detail_pointer/scroll_surface_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/detail_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_control.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/viewport/toolbar/entry.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_route_intent.rs
  - zircon_editor/src/ui/retained_host/tab_drag/bridge.rs
  - zircon_editor/src/tests/host/retained_document_tab_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_drawer_header_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_activity_rail_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_list_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_detail_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_host_page_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_tab_drag/surface_contract.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/route_intent/mod.rs
  - zircon_editor/src/ui/retained_host/route_intent/map.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer
  - zircon_editor/src/ui/retained_host/detail_pointer
  - zircon_editor/src/ui/retained_host/host_page_pointer
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/toolbar.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/route_intent/mod.rs zircon_editor/src/ui/retained_host/route_intent/map.rs
  - cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-check-0623 --message-format short --color never
  - cargo test -p zircon_editor --lib shell_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-shell-pointer-offline-0623 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-check-0623 --message-format short --color never
  - cargo test -p zircon_editor --lib route_intent_only --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_document_tab_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_drawer_header_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_menu_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_activity_rail_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s2-test-0623 --message-format short --color never -- --test-threads=1
  - rustfmt --edition 2021 --check M5.S3 retained-host source/tests
  - M5.S3 source scan for old PointerTarget/map_route/handled_by/route.target/targets remnants
  - cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-check-0623 --message-format short --color never
  - cargo test -p zircon_editor --lib route_intent --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_list_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_detail_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-test-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_host_page_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-focused-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_viewport_toolbar_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-focused-0623 --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib retained_tab_drag --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s3-focused-0623 --message-format short --color never -- --test-threads=1
  - rustfmt --edition 2021 --check M5.S4 viewport toolbar retained-host source/tests
  - M5.S4 source scan for old viewport toolbar surface_hit_test/active control remnants
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s4-integration-0623 --message-format short --color never -- --test-threads=1
  - cargo check -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-realhost-0623 --message-format short --color never
doc_type: module-detail
---

# Retained Host Route Intent

`route_intent/` is the retained-host adapter between runtime UI routing facts and editor semantic actions. The map stores stable `UiRouteId` entries and binds hand-built surface nodes to those routes when a surface is not generated from a compiled UI asset.

The initial consumer was `shell_pointer`. M5.S2 extends the same boundary to document tabs, drawer headers, menus, and the activity rail. M5.S3 adds hierarchy, detail scroll surfaces, host page tabs, viewport toolbar controls, and welcome recent entries. M5.S4 moves viewport-toolbar native clicks off `host_contract/surface_hit_test` entirely: the host callback reports only surface key, point, and size, then `ViewportToolbarPointerBridge` syncs the projected `UiSurfaceFrame` into route-intent-backed runtime nodes before dispatch. Each bridge now builds runtime `UiSurface` nodes, binds the interactive nodes to synthetic route ids, and asks `EditorRouteIntentMap` to resolve the dispatch result into a semantic editor route.

## Current Intents

- `Shell(HostShellPointerRoute)` covers shell drag and resize routes.
- `DocumentTab(HostDocumentTabPointerRoute)` covers tab activation and close buttons.
- `DrawerHeader(HostDrawerHeaderPointerRoute)` covers drawer header tab activation.
- `Menu(HostMenuPointerRouteIntent)` covers menu buttons, submenu branches, leaf item actions, popup surfaces, and dismiss overlay nodes. The private intent keeps submenu item paths that are needed by hover handling before projecting to the public route.
- `ActivityRail(HostActivityRailPointerRoute)` covers rail strips and rail buttons.
- `Hierarchy(HierarchyPointerRoute)` covers the hierarchy list surface and object rows.
- `Detail(ScrollSurfacePointerRoute)` covers retained detail/inspector scroll viewports.
- `HostPage(HostPagePointerRoute)` covers host page tab activation.
- `ViewportToolbar(ViewportToolbarPointerRoute)` covers viewport tool, mode, snap, display, and frame controls.
- `WelcomeRecent(WelcomeRecentPointerRouteIntent)` covers recent-project list rows, open/remove actions, and list-surface scroll. The private intent keeps action path data before projecting to the public route.

## Boundary Rules

- Keep route id and node id lookup in `route_intent/map.rs`.
- Keep bridge-specific route construction in the surface builder for that bridge; the dispatch path should consume the typed `*_route_for_pointer_dispatch(...)` helper instead of matching raw node ids.
- `intent_for(...)` accepts `UiComponentEventReport` for future compiled-template consumers, while node and route lookup cover current hand-built retained-host surfaces.
- Pointer bridges must not read `UiPointerDispatchResult.handled_by` or `UiPointerDispatchResult.route.target` directly. That fallback order belongs in `EditorRouteIntentMap`.
- Do not reintroduce pointer bridge local target tables, node-id `match` blocks, or `host_*_pointer_target` modules.

## Validation Notes

The 2026-06-23 M5.S1 slice added `EditorRouteIntentMap` and migrated `shell_pointer` to consume route intents. The planned locked command is still blocked before Rust diagnostics by current lockfile drift, but the offline focused shell pointer test passed 13/13 and editor lib offline check passed with existing warning noise only.

The 2026-06-23 M5.S2 slice migrated `document_tab_pointer`, `drawer_header_pointer`, `menu_pointer`, and `activity_rail_pointer` to the same adapter. Focused offline route-intent and bridge suites pass. `workbench_projection_cutover` now passes the updated source-hit-test contract but still has one non-M5.S2 failure for the missing dock-header v2 asset reference in production code.

The 2026-06-23 M5.S3 slice migrated `hierarchy_pointer`, `detail_pointer`, `host_page_pointer`, `viewport_toolbar_pointer`, and `welcome_recent_pointer` to route intents and deleted their local target/map route modules. `tab_drag` consumes `HostShellPointerBridge::drag_target_at(point)`, so its contract asserts dependency on the shell route-intent bridge instead of adding a second hit table. `asset_pointer` currently has no independent runtime-surface pointer bridge, and `drawer_resize` remains covered by the shell resize route from M5.S1. Rustfmt, source scans, `cargo check -p zircon_editor --lib --offline`, `route_intent`, `retained_list_pointer`, `retained_detail_pointer`, `retained_host_page_pointer` 8/8, `retained_viewport_toolbar_pointer` 7/7, and `retained_tab_drag` 37/37 pass. The source-contract tests were adjusted to read the current child owner files created by the structure-convention work instead of asserting behavior lives in root `globals.rs`, `callback_wiring.rs`, or `pointer_layout.rs`.

The 2026-06-23 M5.S4 cleanup deleted the old viewport toolbar `surface_hit_test` owner and its test file. Native toolbar routing now uses the submitted `PaneData.viewport.toolbar_surface_frame` with runtime `hit_test_surface_frame(...)` only to preserve the clicked control id for damage; the externally visible callback carries `surface_key + point + size`. `ViewportToolbarPointerBridge::sync_surface_frame(...)` converts projected toolbar controls into route-intent-backed pointer controls, so command dispatch still flows through the shared runtime surface and `EditorRouteIntentMap`. The integration-contract-only `resolve_host_drag_target_group_with_workbench_shell_geometry(...)` keeps crate-external contract tests on componentized workbench geometry without exposing internal layout-frame DTOs. Rustfmt, source scans, editor lib offline check, integration contracts 27/27, and the editor-host app offline compile smoke pass. The planned `--locked` acceptance commands still fail before compilation until the root lockfile drift is resolved; `Cargo.lock` is restored to the protected hash after no-locked/offline validation.
