---
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/workbench/window_registry/menu_overflow_mode.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/tab_drag/host_resolution.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/workbench/document_tabs/metrics.rs
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/compute.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/build.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/reflection/mod.rs
  - zircon_editor/src/ui/reflection/adapter.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tree_row/surface.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/tree.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_editor/src/tests/host/retained_window/native_host_contract.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_drawer_breakpoints.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/layout/drawer_toggle.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/layout/floating_window_focus.rs
  - zircon_editor/src/tests/ui/project_overview/bootstrap_assets.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_popup.zui
  - zircon_editor/assets/ui/editor/workbench_page_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_dock_header.zui
  - zircon_editor/assets/ui/editor/workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/host/editor_main_frame.zui
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.zui
  - zircon_editor/assets/ui/editor/host/floating_window_source.zui
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
implementation_files:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/host_activity_rail_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/host_page_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/tab_drag/host_resolution.rs
  - zircon_editor/src/ui/reflection/mod.rs
  - zircon_editor/src/ui/reflection/adapter.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/workbench/document_tabs/metrics.rs
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/compute.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/build.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_popup.zui
  - zircon_editor/assets/ui/editor/workbench_page_chrome.zui
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/assets/ui/editor/workbench_dock_header.zui
  - zircon_editor/assets/ui/editor/workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/host/editor_main_frame.zui
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.zui
  - zircon_editor/assets/ui/editor/host/floating_window_source.zui
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
plan_sources:
  - user: 2026-05-07 继续里程碑直到完成所有里程碑，主界面表现与 JetBrains/Slate 风格一致
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
  - user: 2026-06-24 editor UI architecture screenshot validation and layout readability pass
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
  - cargo test -p zircon_editor retained_menu_pointer -- --nocapture (2026-05-11: passed, 22 passed, 4 ignored)
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11: passed)
  - cargo test -p zircon_editor retained_activity_rail_pointer -- --nocapture (2026-05-11: passed, 7 passed)
  - cargo test -p zircon_editor --lib shared_activity_rail_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_document_tab_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_drawer_header_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_host_page_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_viewport_toolbar_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib retained_document_tab_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor --lib retained_drawer_header_pointer -- --nocapture (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_editor --lib retained_host_page_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor --lib retained_viewport_toolbar_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor workbench_projection_cutover -- --nocapture (2026-05-11: passed, 9 passed)
  - cargo test -p zircon_editor boundary -- --nocapture (2026-05-11: passed, 72 passed)
  - cargo test -p zircon_editor fallback_page_chrome_preserves_clickable_tab_and_project_path_frames --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-visual-20260521-hostonly --message-format short --color never (2026-05-22: passed)
  - cargo test -p zircon_editor page_and_dock_tabs_project_svg_icons_and_close_button_icon --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-visual-20260521-hostonly --message-format short --color never (2026-05-22: passed)
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-editor-visual-20260521-hostonly --message-format short --color never (2026-05-22: passed)
  - real-window screenshots: target/editor-visual-check/editor-default-after-20260522-034247.png and target/editor-visual-check/editor-material-lab-after-20260522-034247.png
  - real-window screenshots: target/editor-visual-check/editor-default-moveonly-20260522-042143.png and target/editor-visual-check/editor-material-lab-topmost2-20260522-035453.png
  - temporary stack probe screenshot: target/editor-visual-check/editor-default-960x640-stack8m-20260522-043217.png
  - rebuilt stack validation screenshot: target/editor-visual-check/editor-default-960x640-rebuilt-stack8m-20260522-043929.png
  - E:\cargo-targets\zircon-editor-layout-visual-fix-0624\debug\deps\zircon_editor-820618fe5427109a.exe tests::host::retained_menu_pointer::visual_screenshot::capture_m3_gui_acceptance_visual_artifacts --ignored --exact --test-threads=1 --nocapture (2026-06-24: passed, refreshed 8 screenshots under `target/visual-layout` at 09:43 +08)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --message-format short --color never (2026-06-24 09:59 +08: passed with existing warning noise)
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --message-format short --color never -- --test-threads=1 (2026-06-24: timed out after 20 minutes during compile; not counted as passing)
  - CARGO_INCREMENTAL=0 cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --quiet (2026-06-24: passed)
  - CARGO_INCREMENTAL=0 cargo test -p zircon_editor --lib --locked --no-run --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --quiet (2026-06-24: passed)
  - direct zircon_editor test binary capture_m3_gui_acceptance_visual_artifacts --ignored --test-threads=1 --nocapture (2026-06-24: passed, refreshed 8 screenshots under `docs/tests/editor`)
  - direct zircon_editor test binary declared_workbench_module_events_dispatch_preview_actions --test-threads=1 --nocapture (2026-06-24: passed)
  - cargo fmt --all (2026-06-25: passed after Workbench projection source fix)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --quiet (2026-06-25: passed with existing warning noise)
  - python .codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\audit_editor_structure.py --json (2026-06-25: passed, `m1_gate_status=classified-and-clear`, structure debt counts 0)
  - cargo test -p zircon_editor --lib workbench_projection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 -- --test-threads=1 (2026-06-25: blocked before target tests by rustc 1.94.1 Windows ICE in `zircon_runtime_interface` metadata encoding)
  - cargo test -p zircon_editor --lib dock_header_nodes_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 -- --test-threads=1 (2026-06-25: blocked before target tests by the same rustc ICE)
  - wsl cargo test -p zircon_editor --lib workbench_projection --locked --jobs 1 --target-dir /tmp/zircon-editor-ui-wsl-0625 -- --test-threads=1 --nocapture (2026-06-25: timed out after 20 minutes while compiling `zircon_runtime`; stopped residual WSL cargo/rustc processes; not counted as passing)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never (2026-06-25: passed with existing warning noise)
  - cargo test -p zircon_editor --lib fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-25: passed)
  - cargo test -p zircon_editor --lib fallback_page_chrome_wide_tier_does_not_force_overflow_when_tabs_fit --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-25: passed)
  - cargo test -p zircon_editor --lib narrow_tier_caps_visible_tabs_before_overflow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-25: passed)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-25: passed, refreshed 640/900/1260 screenshots under `docs/tests/editor`)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never (2026-06-25: passed with existing warning noise)
  - cargo test -p zircon_editor --lib document_tab --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 23 passed)
  - cargo test -p zircon_editor --lib template_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 33 passed)
  - cargo test -p zircon_editor --lib template_icon_buttons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 13 passed)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, refreshed `docs/tests/editor`)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never (2026-06-26: passed with existing warning noise)
doc_type: module-detail
---

# Workbench Main Interface Entries

## Purpose

M3.1a fixes the ownership map for the editor main interface before more GUI behavior is changed. The workbench now presents through the Rust-owned retained host, and the main visible chrome must be backed by `.ui.toml` template assets and shared surface projection. The host may translate platform events, paint template nodes, and copy data into boundary DTOs; it must not become a second source of menu, drawer, toolbar, document pane, or floating-panel geometry.

This document records the current accepted entry map for the M3 host cutover work.

## Entry Map

| Interface area | Canonical entry | Shared surface responsibility | Host responsibility |
|---|---|---|---|
| Top menu bar | `workbench_menu_chrome.zui` through `menu_chrome_nodes(...)` | Author top-row controls and stable `control_id` frames such as `WorkbenchMenuTopBar` and `MenuSlot*` | Inject current menu labels and expose hit frames at the host boundary |
| Menu popup | `workbench_menu_popup.zui` through `menu_popup_nodes(...)` | Author popup rows, label/shortcut slots, icon-bearing menu item stencils | Expand rows beyond authored stencils, clamp popup in M3/M4, dispatch menu action bindings |
| Page chrome | `workbench_page_chrome.zui` through `page_chrome_nodes(...)` | Author page strip, project-path label, and page tab frames | Copy current page data and expose tab hit/drag frames |
| Document, side, bottom, floating headers | `workbench_dock_header.zui` through dock-header projection functions | Author common tab/header shape, close buttons, subtitle frames and icon metadata | Project document/side/bottom/floating tab data without a per-pane hit table |
| Status bar | `workbench_status_bar.zui` through `status_bar_nodes(...)` | Author status labels and viewport label frames | Inject status text only |
| Activity rail | `workbench_activity_rail.zui` through `activity_rail_nodes(...)` | Author activity buttons and icon stencils | Copy active tab state and icon keys |
| Main shell and drawers | `host/workbench_shell.zui` plus `windows/workbench_window.zui` and the Workbench shell components | Author stable workbench regions, real drawer shell/header/content roots, document host root and dock routes | Fill `HostWindowSurfaceData` from `ShellPresentation::from_state(...)` and keep drawer frame constraints inside the Workbench window bridge |
| Floating panel source | `host/floating_window_source.zui` plus `collect_floating_windows(...)` | Author floating panel source structure | Place native floating windows and keep current activation state |
| Scene toolbar | `host/scene_viewport_toolbar.zui` through pane presentation | Author the toolbar as a template-backed pane body/control band | Bind current scene settings and dispatch toolbar actions |

## Behavior Model

`chrome_template_projection.rs` is the current template projection boundary for shared workbench chrome. It keeps the source-contract constants and thin wrappers, while `chrome_template_projection/menu_chrome.rs` owns menu chrome/popup projection and `chrome_template_projection/dock_header.rs` owns the document, side, bottom, and floating dock-header projection. These owners load the root chrome assets with `build_view_template_nodes(...)`, apply current labels, tab state, icon names, and disabled/selected state, then return `ViewTemplateNodeData` rows. Fallback nodes are allowed only as resilience for missing template metrics; they are not a new business UI source. The fallback page chrome must preserve the same row split as the retained shell: menu bar first, page bar below `MENU_TOP_BAR_HEIGHT_PX`, then document/content tabs. Fallback icon nodes publish `icons/ionicons/*.svg` media sources so the retained painter and runtime asset resolver agree with the crate-local icon directory.

Dock-header document tabs now use `ui/workbench/document_tabs/metrics.rs` as the shared geometry owner. `chrome_template_projection/dock_header.rs` uses it for readable document-tab width, strip offset, gap, tab height, and close-button placement, while `document_tab_pointer/constants.rs` aliases the same values for hit testing. This keeps the visible `Asset Browser` document tab and its close button aligned with the retained pointer surface instead of maintaining separate paint and hit constants.

The viewport gizmo axis labels keep their authored colors through the Workbench viewport ZUI owner: `workbench_viewport_panel.zui` declares explicit `style.self.foreground_color` values for X/Y/Z, and Workbench projection keeps command-control text separate from value-display text. Tree rows and Inspector property rows use the same rule at the surface layer: selected rows consume declared `background_color` or dynamic `background`/`border` aliases instead of falling back to the default selected surface.

`shell_presentation.rs` creates one `HostWindowSurfaceData` value from `WorkbenchViewModel`, `EditorChromeSnapshot`, and current geometry. The DTO contains the host tabs, drawer tabs, document tabs, floating windows, side panes, bottom pane, and document pane in one place, so later host rendering consumes a single projected surface packet instead of reaching back into workbench state from separate widgets.

`pane_projection.rs` picks the active drawer/document tab and delegates body construction to `PanePresentation`. Pane headers and bodies keep the same entry path: state is collected in Rust, but the visible control structure comes from `.ui.toml` assets or shared pane presentation builders.

`reflection.rs` exposes the same shape to the UI reflection tree: `MenuBar`, `PageCollection`, `DrawerCollection`, and `FloatingWindows` are shared reflection nodes, while activities report whether they are hosted by a drawer, document page, floating window, or exclusive page. This prevents debug tooling from having to infer editor structure from host-only paint data.

## Pointer Boundary

M3.1b keeps the remaining host pointer paths as shared surface bridges instead of host-only hit tables. The accepted bridge set is:

- `menu_pointer`: owns menu button, popup, dismissal, scroll and item routes through `UiSurface`, `UiTreeNode` and `UiPointerDispatcher`. Matching layout and state sync input preserves the current menu pointer surface, while scroll, hover, popup, and submenu state changes still rebuild.
- `activity_rail_pointer`: turns left/right rail buttons into a shared surface and routes tab activation through dispatcher effects. Its sync path skips surface rebuilds when the new rail layout equals the committed layout, so repeated projection syncs do not churn the pointer surface during pointer-heavy frames.
- `host_page_pointer`: turns the main page strip into shared tab hit nodes; matching sync input preserves measured tab frames and the current pointer surface.
- `drawer_header_pointer`: builds drawer header tab surfaces from current layout and measured frames, but still dispatches through shared pointer nodes. Matching sync input preserves the current measured-frame cache and pointer surface.
- `document_tab_pointer`: builds document tab and close-button surfaces, including measured/fallback tab frames, then dispatches through the shared pointer dispatcher. Matching sync input preserves the current measured-frame cache and pointer surface.
- `viewport_toolbar_pointer`: projects active toolbar controls into a surface, keeping action routing separate from paint code. Matching sync input preserves active-control state and the current pointer surface.
- `shell_pointer`: owns drag and resize surfaces for host-level window movement, drawer resize and document docking; it uses shared dispatch instead of direct rectangle tables.
- `host_contract/surface_hit_test/template_node.rs`: converts template pane nodes into a `UiSurfaceFrame` for binding/callback hit testing, then resolves hits through the same surface-frame helper used by host hit tests.

The only production change in this slice was making `tab_drag/host_resolution.rs` prefer the canonical `ActivityDrawerSlot::Bottom` for the public bottom group while still accepting legacy `BottomLeft` and `BottomRight`. This keeps M3 and the drawer/window/menu migration aligned with the registry contract that bottom drawers have a single public position.

M4 menu overflow keeps this same pointer boundary. `menu_pointer/popup_layout.rs` now resolves popup bounds and optional multi-column geometry for the shared pointer surface, while `host_menu_pointer_bridge_rebuild_surface.rs` places item nodes by column and row without changing absolute item indices. Popup width/height are capped to the shell before hit nodes are emitted, so constrained hosts do not leave interactive menu rows outside the shell. The visual/menu chrome projection may still be documented in template-runtime docs, but pointer hit testing, scroll state, right-edge clamp behavior, action routing, and identical sync early-return behavior remain owned by `HostMenuPointerBridge`.

## Edge Cases and Constraints

- Root workbench chrome v2 assets under `assets/ui/editor/` are the canonical chrome projection inputs. The old root `.ui.toml` files and any host-folder duplicates are migration artifacts only; `chrome_template_projection.rs` must keep pointing at the v2 root chrome assets.
- Host `.ui.toml` assets under `assets/ui/editor/host/` are allowed for native window shell, floating source, toolbar, and pane body projection. Drawer frame owners now live in the real Workbench window/component assets; deleted host drawer-source assets and embedded frame-only drawer projection must not return as generated UI business source files.
- The host must not introduce menu, drawer, floating-window, document-pane, or toolbar hit tables. Hit frames should come from template node frames or shared surface hit data.
- Fallback chrome nodes must preserve clickable frames and icon metadata. They are guardrails for a failed template load, not a parallel design system.

## Test Coverage

`workbench_main_interface_entries_are_template_backed_and_reflected` statically verifies the M3.1a entry map. It checks that the canonical chrome and host template assets exist, that chrome projection references the root chrome assets and shared `build_view_template_nodes(...)` path, that shell/pane projection owns the expected single DTO entry points, and that `reflection.rs` exposes the shared workbench tree.

`workbench_host_pointer_paths_are_shared_surface_bridges_not_host_hit_tables` verifies M3.1b. It scans the active editor host tree for deleted generated UI business files and forbidden hit-table names, then checks that the menu, activity rail, drawer header, document tab, viewport toolbar, shell drag/resize and template-node hit paths all keep `UiSurface` / `UiPointerDispatcher` / `UiSurfaceFrame` ownership.

M3.2 now has focused interaction gates:

- `native_host_generic_template_text_field_routes_commit_binding_on_enter` proves text input keeps edit binding on text insertion and switches to the commit binding on Enter.
- `shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model` builds a nested `MenuItemModel::branch`, flattens the leaf through the shared menu pointer layout, clicks the leaf row, and verifies the EditorOperation runtime receives `weather.cloud_layer.refresh`.
- `shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell` proves the shared popup hit surface caps width and height to a tiny shell instead of leaving hit frames outside the host bounds.
- `shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp` opens a small-shell right-edge popup in `MenuOverflowMode::MultiColumn`, clicks the first row in the second column, and verifies the shared pointer route keeps absolute item index `9` after horizontal clamp.
- `builtin_host_activity_toggle_*` covers drawer close and reopen through the template bridge. These tests use global harness state and must be batched with `--test-threads=1`.
- `tab_drop_dispatch_*` covers drawer attach, collapsed drawer reopen, auto-hide preservation, document edge split, and detached drawer-window routes.
- `builtin_floating_window_focus_*` covers floating-window activation, source filtering, redundant focus skipping, and parity with the legacy focus event log.

M3.3 adds the screenshot gate `capture_m3_gui_acceptance_visual_artifacts`. It writes the accepted GUI artifact set under `docs/tests/editor`: Welcome input, Workbench, standalone Asset Browser, embedded Assets drawer, menu popup with SVG icons, drag-after-release, and small/large SVG scaling captures. These screenshots closed visual regressions that focused unit tests could not catch: default schema `text = "Button"` leaking into authored buttons, embedded asset drawer chip clipping, and 900x620 Workbench document collapse.

The 2026-06-24 validation reran the same gate against the retained-host Workbench at 900x620, 640x420, and 1260x780. It also locks the Project Overview drawer: the left and right drawer compact width inputs now preserve the central Scene viewport, while the `Open Assets` and `Asset Browser` buttons remain fully readable in the 900x620 Workbench screenshot and crop artifact. The final closeout adds a 900x620 reference Workbench assertion for document width, viewport containment, and status-bar bottom anchoring; the refreshed `docs/tests/editor/editor-window-m3-*.png` set is the accepted layout evidence for this slice.

The label leak is covered by `render_extract_uses_label_when_schema_text_default_is_placeholder` and `project_overview_projection_maps_bootstrap_asset_into_template_nodes`. Runtime render extraction now prefers visible authored labels for text-bearing controls and keeps `IconButton` labels accessibility-only. The editor projection layer follows the same rule when building host template node data.

The final M3.T batch reruns the editor focused gates plus SVG target-size and tint regressions: `svg_icon_pixels_follow_requested_target_size`, `template_icon_tint_uses_material_state_priority`, workbench entry/pointer projection tests, and `workbench_projection` all passed with the refreshed screenshots. Existing runtime/editor warning noise is not caused by this module.

The 2026-06-25 focused source audit keeps the refreshed `docs/tests/editor` screenshots as the current layout evidence, but it does not claim fresh `workbench_projection` or `dock_header_nodes_` execution as passed. Both Windows Cargo filters stopped during `zircon_runtime_interface` dependency compilation with a rustc 1.94.1 ICE in `rustc_metadata` before the target tests ran. A WSL same-toolchain probe avoided an immediate ICE but timed out after 20 minutes while still compiling `zircon_runtime`, and its residual cargo/rustc processes were stopped. The open validation item is a clean long-window or CI-parity rerun that reaches actual test results.

The 2026-06-25 S15.5a breakpoint pass adds a shared Workbench layout tier owner. `layout_tier.rs` classifies the reference screenshot widths as 640=Narrow, 900=Regular, and 1260=Wide; `compute_workbench_shell_geometry(...)` forces the right tool region through the existing collapsed rail constraint only in the Narrow tier. The componentized Workbench template bridge consumes the same tier rule in `drawer_layout.rs`, so the right drawer shell/content roots are hidden at 640 instead of drawing a full Inspector against a rail-sized geometry. `narrow_workbench_geometry_collapses_right_drawer_to_rail`, `workbench_layout_tiers_classify_reference_capture_widths`, and `componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width` cover the geometry and bridge paths. The refreshed `docs/tests/editor/editor-window-m3-svg-icon-scale-small-640x420.png` shows the right Inspector content folded away, while `editor-window-m3-svg-icon-scale-large-1260x780.png` keeps the right drawer visible. This closes only the first drawer breakpoint slice; page-tab overflow, table-column narrow-tier policy, and tokenized breakpoint defaults remain open.

The next S15.3/S15.5 page-chrome pass routes fallback page chrome through the same page-tab component metrics as the host pointer path. `ui/workbench/page_tabs/metrics.rs` now owns the project-path reserve, overflow-popup width, and `LayoutTier` visible-tab cap. `chrome_template_projection.rs` uses those values before it lays out fallback page tabs, so Narrow tier keeps at most two visible main page tabs plus overflow while reserving project path space, and Wide tier does not force overflow when all tabs fit. `fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path`, `fallback_page_chrome_wide_tier_does_not_force_overflow_when_tabs_fit`, and `narrow_tier_caps_visible_tabs_before_overflow` cover the projection and retained pointer paths. The refreshed 640/900/1260 screenshots under `docs/tests/editor` are the current visual evidence for this page-tab breakpoint linkage, and the final `zircon_editor` build passed from `D:\cargo-targets\zircon-editor-components-0625`.

The 2026-05-22 visual validation pass reproduced a real-window overlap in the procedural page-chrome fallback: page tabs started at `y = 0`, so they occupied the same vertical band as the top menu. `fallback_page_chrome_preserves_clickable_tab_and_project_path_frames` now locks the fallback page bar below `MENU_TOP_BAR_HEIGHT_PX`, verifies tab/project-path frames stay inside that bar, and `page_and_dock_tabs_project_svg_icons_and_close_button_icon` verifies projected tab icons still resolve through `icons/ionicons/*.svg`. The acceptance artifacts are the real-window default and Material-lab screenshots listed in the document header; both were sampled as nonblank and showed menu/page/content rows separated.

The same pass exposed a Windows/MSVC host process stack-budget issue outside the chrome projection itself: the default UI Component Showcase stayed stable at the initial 1280 x 720 size, but the unmodified binary exited during a real 960 x 640 resize before committing the second presentation. A temporary PE-header probe with an 8 MB `zircon_editor` stack survived the same resize and produced the listed 960 x 640 screenshot. The durable fix is documented in `docs/zircon_app/editor-host-entry.md` and implemented by `zircon_app/build.rs`; the rebuilt source binary reports `800000 size of stack reserve` and survived the same 960 x 640 default resize screenshot probe.

## Plan Sources

This document closes the inventory portion of M3.1a from `Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md` and aligns it with the main-frame/drawer/menu sequencing described in `Drawer_Window_Menu Slate 化推进计划.md`.

## Open Issues or Follow-up

- M3 is accepted. Follow-up visual work moves to M4/M8: editor/runtime same `.ui.toml` golden or paint snapshots, final GUI screenshot bundle, and cleanup of duplicate host/root chrome assets.
