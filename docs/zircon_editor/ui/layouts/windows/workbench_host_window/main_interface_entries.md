---
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/workbench/window_registry/menu_overflow_mode.rs
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/host_document_tab_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/tab_drag/host_resolution.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/reflection.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/drawer_toggle.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/floating_window_focus.rs
  - zircon_editor/src/tests/ui/project_overview/bootstrap_assets.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_editor/assets/ui/editor/asset_browser.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
  - zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
implementation_files:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/host_document_tab_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/tab_drag/host_resolution.rs
  - zircon_editor/src/ui/reflection.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
  - zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
plan_sources:
  - user: 2026-05-07 继续里程碑直到完成所有里程碑，主界面表现与 JetBrains/Slate 风格一致
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
tests:
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - cargo test -p zircon_editor --lib workbench_main_interface_entries_are_template_backed_and_reflected --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib workbench_host_pointer_paths_are_shared_surface_bridges_not_host_hit_tables --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_generic_template_text_field_routes_commit_binding_on_enter --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib builtin_host_activity_toggle --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib tab_drop_dispatch_ --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib builtin_floating_window_focus --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --ignored --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib template_icon_tint_uses_material_state_priority --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_extract_uses_label_when_schema_text_default_is_placeholder --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib project_overview_projection_maps_bootstrap_asset_into_template_nodes --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Workbench Main Interface Entries

## Purpose

M3.1a fixes the ownership map for the editor main interface before more GUI behavior is changed. The workbench still presents through the native/Slint host, but the main visible chrome must be backed by `.ui.toml` template assets and shared surface projection. The host may translate platform events, paint template nodes, and copy data into boundary DTOs; it must not become a second source of menu, drawer, toolbar, document pane, or floating-panel geometry.

This document records the current accepted entry map for the M3 host cutover work.

## Entry Map

| Interface area | Canonical entry | Shared surface responsibility | Host responsibility |
|---|---|---|---|
| Top menu bar | `workbench_menu_chrome.ui.toml` through `menu_chrome_nodes(...)` | Author top-row controls and stable `control_id` frames such as `WorkbenchMenuTopBar` and `MenuSlot*` | Inject current menu labels and expose hit frames at the host boundary |
| Menu popup | `workbench_menu_popup.ui.toml` through `menu_popup_nodes(...)` | Author popup rows, label/shortcut slots, icon-bearing menu item stencils | Expand rows beyond authored stencils, clamp popup in M3/M4, dispatch menu action bindings |
| Page chrome | `workbench_page_chrome.ui.toml` through `page_chrome_nodes(...)` | Author page strip, project-path label, and page tab frames | Copy current page data and expose tab hit/drag frames |
| Document, side, bottom, floating headers | `workbench_dock_header.ui.toml` through dock-header projection functions | Author common tab/header shape, close buttons, subtitle frames and icon metadata | Project document/side/bottom/floating tab data without a per-pane hit table |
| Status bar | `workbench_status_bar.ui.toml` through `status_bar_nodes(...)` | Author status labels and viewport label frames | Inject status text only |
| Activity rail | `workbench_activity_rail.ui.toml` through `activity_rail_nodes(...)` | Author activity buttons and icon stencils | Copy active tab state and icon keys |
| Main shell and drawers | `host/workbench_shell.ui.toml` and `host/workbench_drawer_source.ui.toml` | Author stable workbench regions, drawer roots, document host root and dock routes | Fill `HostWindowSurfaceData` from `ShellPresentation::from_state(...)` |
| Floating panel source | `host/floating_window_source.ui.toml` plus `collect_floating_windows(...)` | Author floating panel source structure | Place native floating windows and keep current activation state |
| Scene toolbar | `host/scene_viewport_toolbar.ui.toml` through pane presentation | Author the toolbar as a template-backed pane body/control band | Bind current scene settings and dispatch toolbar actions |

## Behavior Model

`chrome_template_projection.rs` is the current template projection boundary for shared workbench chrome. It loads the root chrome assets with `build_view_template_nodes(...)`, applies current labels, tab state, icon names, and disabled/selected state, then returns `ViewTemplateNodeData` rows. Fallback nodes are allowed only as resilience for missing template metrics; they are not a new business UI source.

`shell_presentation.rs` creates one `HostWindowSurfaceData` value from `WorkbenchViewModel`, `EditorChromeSnapshot`, and current geometry. The DTO contains the host tabs, drawer tabs, document tabs, floating windows, side panes, bottom pane, and document pane in one place, so later host rendering consumes a single projected surface packet instead of reaching back into workbench state from separate widgets.

`pane_projection.rs` picks the active drawer/document tab and delegates body construction to `PanePresentation`. Pane headers and bodies keep the same entry path: state is collected in Rust, but the visible control structure comes from `.ui.toml` assets or shared pane presentation builders.

`reflection.rs` exposes the same shape to the UI reflection tree: `MenuBar`, `PageCollection`, `DrawerCollection`, and `FloatingWindows` are shared reflection nodes, while activities report whether they are hosted by a drawer, document page, floating window, or exclusive page. This prevents debug tooling from having to infer editor structure from host-only paint data.

## Pointer Boundary

M3.1b keeps the remaining host pointer paths as shared surface bridges instead of host-only hit tables. The accepted bridge set is:

- `menu_pointer`: owns menu button, popup, dismissal, scroll and item routes through `UiSurface`, `UiTreeNode` and `UiPointerDispatcher`.
- `activity_rail_pointer`: turns left/right rail buttons into a shared surface and routes tab activation through dispatcher effects.
- `drawer_header_pointer`: builds drawer header tab surfaces from current layout and measured frames, but still dispatches through shared pointer nodes.
- `document_tab_pointer`: builds document tab and close-button surfaces, including measured/fallback tab frames, then dispatches through the shared pointer dispatcher.
- `viewport_toolbar_pointer`: projects active toolbar controls into a surface, keeping action routing separate from paint code.
- `shell_pointer`: owns drag and resize surfaces for host-level window movement, drawer resize and document docking; it uses shared dispatch instead of direct rectangle tables.
- `host_contract/surface_hit_test/template_node.rs`: converts template pane nodes into a `UiSurfaceFrame` for binding/callback hit testing, then resolves hits through the same surface-frame helper used by host hit tests.

The only production change in this slice was making `tab_drag/host_resolution.rs` prefer the canonical `ActivityDrawerSlot::Bottom` for the public bottom group while still accepting legacy `BottomLeft` and `BottomRight`. This keeps M3 and the drawer/window/menu migration aligned with the registry contract that bottom drawers have a single public position.

M4 menu overflow keeps this same pointer boundary. `menu_pointer/popup_layout.rs` now resolves popup bounds and optional multi-column geometry for the shared pointer surface, while `host_menu_pointer_bridge_rebuild_surface.rs` places item nodes by column and row without changing absolute item indices. Popup width/height are capped to the shell before hit nodes are emitted, so constrained hosts do not leave interactive menu rows outside the shell. The visual/menu chrome projection may still be documented in template-runtime docs, but pointer hit testing, scroll state, right-edge clamp behavior, and action routing remain owned by `HostMenuPointerBridge`.

## Edge Cases and Constraints

- Root workbench chrome assets under `assets/ui/editor/` are the canonical chrome projection inputs. Host-folder duplicates may remain as migration artifacts, but `chrome_template_projection.rs` must point to the root chrome assets until M8 cleanup removes duplicate sources.
- Host `.ui.toml` assets under `assets/ui/editor/host/` are allowed for native window shell, drawer source, floating source, toolbar, and pane body projection. They are still template assets, not `.slint` business UI files.
- The host must not introduce menu, drawer, floating-window, document-pane, or toolbar hit tables. Hit frames should come from template node frames or shared surface hit data.
- Fallback chrome nodes must preserve clickable frames and icon metadata. They are guardrails for a failed template load, not a parallel design system.

## Test Coverage

`workbench_main_interface_entries_are_template_backed_and_reflected` statically verifies the M3.1a entry map. It checks that the canonical chrome and host template assets exist, that chrome projection references the root chrome assets and shared `build_view_template_nodes(...)` path, that shell/pane projection owns the expected single DTO entry points, and that `reflection.rs` exposes the shared workbench tree.

`workbench_host_pointer_paths_are_shared_surface_bridges_not_host_hit_tables` verifies M3.1b. It scans the active editor host tree for deleted business `.slint` files and forbidden hit-table names, then checks that the menu, activity rail, drawer header, document tab, viewport toolbar, shell drag/resize and template-node hit paths all keep `UiSurface` / `UiPointerDispatcher` / `UiSurfaceFrame` ownership.

M3.2 now has focused interaction gates:

- `native_host_generic_template_text_field_routes_commit_binding_on_enter` proves text input keeps edit binding on text insertion and switches to the commit binding on Enter.
- `shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model` builds a nested `MenuItemModel::branch`, flattens the leaf through the shared menu pointer layout, clicks the leaf row, and verifies the EditorOperation runtime receives `Weather.CloudLayer.Refresh`.
- `shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell` proves the shared popup hit surface caps width and height to a tiny shell instead of leaving hit frames outside the host bounds.
- `shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp` opens a small-shell right-edge popup in `MenuOverflowMode::MultiColumn`, clicks the first row in the second column, and verifies the shared pointer route keeps absolute item index `9` after horizontal clamp.
- `builtin_host_activity_toggle_*` covers drawer close and reopen through the template bridge. These tests use global harness state and must be batched with `--test-threads=1`.
- `tab_drop_dispatch_*` covers drawer attach, collapsed drawer reopen, auto-hide preservation, document edge split, and detached drawer-window routes.
- `builtin_floating_window_focus_*` covers floating-window activation, source filtering, redundant focus skipping, and parity with the legacy focus event log.

M3.3 adds the screenshot gate `capture_m3_gui_acceptance_visual_artifacts`. It writes the accepted GUI artifact set under `target/visual-layout`: Welcome input, Workbench, standalone Asset Browser, embedded Assets drawer, menu popup with SVG icons, drag-after-release, and small/large SVG scaling captures. These screenshots closed two visual regressions that focused unit tests could not catch: default schema `text = "Button"` was leaking into authored buttons, and the embedded asset drawer was clipping chip labels inside the compact left tool window.

The label leak is covered by `render_extract_uses_label_when_schema_text_default_is_placeholder` and `project_overview_projection_maps_bootstrap_asset_into_template_nodes`. Runtime render extraction now prefers visible authored labels for text-bearing controls and keeps `IconButton` labels accessibility-only. The editor projection layer follows the same rule when building host template node data.

The final M3.T batch reruns the editor focused gates plus SVG target-size and tint regressions: `svg_icon_pixels_follow_requested_target_size`, `template_icon_tint_uses_material_state_priority`, workbench entry/pointer projection tests, and `workbench_projection` all passed with the refreshed screenshots. Existing runtime/editor warning noise is not caused by this module.

## Plan Sources

This document closes the inventory portion of M3.1a from `Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md` and aligns it with the main-frame/drawer/menu sequencing described in `Drawer_Window_Menu Slate 化推进计划.md`.

## Open Issues or Follow-up

- M3 is accepted. Follow-up visual work moves to M4/M8: editor/runtime same `.ui.toml` golden or paint snapshots, final GUI screenshot bundle, and cleanup of duplicate host/root chrome assets.
