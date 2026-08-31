---
related_code:
  - zircon_editor/src/ui/retained_host/menu_popup_contract.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_clamp_popup_scroll_offset.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_click.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_move.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_route.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_state.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_route_intent.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_item_spec.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/node_ids.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/core/commands/menu_model.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/workbench/window_registry/menu_overflow_mode.rs
  - zircon_editor/src/ui/workbench/window_registry/window_instance.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/menu_bar/mod.rs
  - zircon_editor/src/ui/workbench/menu_bar/metrics.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_popup.zui
implementation_files:
  - zircon_editor/src/ui/retained_host/menu_popup_contract.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_clamp_popup_scroll_offset.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_route.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_state.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_route_intent.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/popup_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_components.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/menu_bar/metrics.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.zui
  - zircon_editor/assets/ui/editor/workbench_menu_popup.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/engine-code-structure-convention.md
  - user: 2026-05-06 Drawer/Window/Menu Slate plan requested menu tree, popup bounds, scroll overflow, and optional multi-column popup behavior
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
tests:
  - shared_menu_pointer_layout_content_measures_root_popup_widths
  - exact production menu-popup contract standalone Rust test entry (2026-07-10: 6/6 passed)
  - cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 (2026-07-10: passed)
  - focused responsive-menu Cargo groups (2026-07-10: 15/15 passed)
  - capture_m3_gui_acceptance_visual_artifacts (2026-07-10: 1/1 passed)
  - zircon_editor/src/tests/host/retained_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/support.rs
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11: passed)
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never
  - cargo test -p zircon_editor --lib menu_overflow_preference --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib retained_menu_pointer --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_opens_flipped_nested_popup_for_branch_hover --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib rust_owned_host_painter_draws_open_nested_menu_popup --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib retained_menu_pointer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture --test-threads=1
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never
  - cargo test -p zircon_editor --lib capture_nested_menu_popup_visual_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --ignored --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib retained_menu_pointer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never -- --nocapture
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never
  - cargo test -p zircon_editor --lib menu_chrome_nodes_project_extension_slots_beyond_authored_stencil --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never -- --nocapture
  - rustfmt --edition 2021 --check zircon_editor/src/ui/workbench/menu_bar/mod.rs zircon_editor/src/ui/workbench/menu_bar/metrics.rs zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs zircon_editor/src/tests/host/retained_menu_pointer/layout.rs zircon_editor/src/tests/host/retained_menu_pointer/support.rs
  - static scan: menu touched paths contain no `chars().count()`, `TITLE_WIDTH_PER_CHAR`, `WIDTH_PER_CHAR`, or `* 7.0` text-width heuristics
  - docs/tests/runtime/text/runtime_text_editor_menu_bar_runtime_measure_preview_20260704.png
  - cargo test -p zircon_editor retained_menu_pointer -- --nocapture (2026-05-11: passed, 22 passed, 4 ignored)
doc_type: module-detail
---

# Host Menu Pointer Bridge

## Purpose

`menu_pointer` is the editor host bridge that turns the projected workbench menu bar into a shared `UiSurface` for pointer routing. It owns the transient hit surface for top-level menu buttons, popup rows, popup dismissal, popup scroll state, and menu item action routing. The host can paint menu chrome elsewhere, but click, hover, dismiss, and scroll decisions should continue to come from this shared surface instead of a host-only rectangle table.

## Data Flow

`build_host_menu_pointer_layout(...)` receives the current `MenuBarModel`, chrome snapshot, shell size, layout presets, and optional projected root-shell frames. It resolves the top-level menu button frames from the shared menu bar or shell frame and builds `HostMenuPointerLayout`. The shared menu bar frame stencil comes from `workbench_menu_chrome.zui` through the v2 file cache rather than reparsing the old root chrome asset. The bridge keeps one normalized stencil keyed by the complete view-template resource generation, so stable pointer-surface recomputation and shell resize translate the cached frames instead of projecting the ZUI again; a template, design-token, or font generation change replaces that entry. The stencil supplies origin, height, and gap; live width comes from `measure_runtime_text_width(..., WORKBENCH_MENU_SLOT_FONT_SIZE)` plus `workbench_menu_slot_width_from_label_width(...)`, so pointer hitboxes match the visual menu slot width for the actual glyphs.

`HostMenuPointerLayout` carries editor action state, preset rows, popup height, explicit tree-shaped `menus`, dynamic top-level button frames, menu-bar content width, menu-bar scroll offset state, and `menu_overflow_mode`. `build_host_menu_pointer_layout(...)` reads overflow mode from `EditorChromeSnapshot`, whose active-window value comes from persisted `ActivityWindowLayout.menu_overflow_mode`. When `menus` is present, the pointer bridge consumes those rows rather than rebuilding hard-coded menu contents. Branch rows stay as enabled tree nodes with children; leaves preserve either legacy `MenuAction` ids or `EditorOperationPath` ids so extension operations dispatch through the operation runtime.

The 2026-07-10 responsive root-menu pass adds `popup_widths` to that layout. Each root width is measured from the same visible labels, branch chevrons, shortcuts, preset names, and active marker projected into native chrome, using the retained runtime text service plus the shared trailing-glyph guard. `popup_layout.rs` consumes the projected width before shell clamp and optional multi-column calculation; the old per-index width table is now only a minimum/fallback for empty or legacy fixtures.

`HostMenuPointerBridge::sync(...)` first compares the incoming layout and state with the committed pair. If both are unchanged, it returns `false` and keeps the current `UiSurface`; otherwise it clamps retained popup and menu-bar scroll offsets against the current metrics, rebuilds the surface, and registers each interactive node with `UiPointerDispatcher`. Clicks on menu buttons open or close the active popup; hover/click on `SubmenuBranch` nodes stores `open_submenu_path` and rebuilds child popup layers; clicks on interactive leaf nodes close the popup and return the action id; clicks outside the popup hit the dismiss overlay.

## Popup Geometry

`popup_layout.rs` is the single geometry helper for the shared pointer surface. It resolves a `PopupGridLayout` containing the popup frame, content frame, rows per column, column width, row step, scroll offset, viewport extent, and content extent.

The default `MenuOverflowMode::Auto` path preserves the existing single-column behavior. Non-window menus use their full content height when they fit, while oversized popups are clamped by shell height and get a scrollable shared surface. Window menus keep using `window_popup_height` so preset lists scroll predictably.

`MenuOverflowMode::MultiColumn` is an explicit active-window preference. Older or unset layouts default to `Auto` through serde defaults, while project/global layout persistence can store `MultiColumn` on the activity window and have it flow through chrome into the production menu pointer layout. In `MultiColumn` mode, `popup_column_count(...)` uses the available height around the anchor to decide how many rows fit in a column, then caps the requested column count by shell width. `host_menu_pointer_bridge_rebuild_surface.rs` places item nodes by column and row while preserving absolute item indices, so a click in column two still routes to the original row index in `HostMenuPointerLayout.menus`.

The popup width is capped by the shell width before x is clamped back into the shell frame. Popup height is capped by the shell height after comparing space below and above the anchor, so even a shell smaller than the normal minimum popup height does not produce hit nodes outside the shell. The y position opens below the menu button when possible and flips above the anchor when the clamped height would otherwise overflow the bottom of the shell.

Nested submenu popups use the hovered branch row as their anchor. They try to open to the right of the parent popup, flip left when the right edge would overflow, and then clamp back inside the shell edge margin. Node ids include the popup level, while public routes keep the legacy pre-order item index. This preserves existing action dispatch and lets the native painter show child popups without collapsing the data back into a flat visual wrapper.

## Scroll And Hover

All popup scroll handling uses `popup_scroll_metrics(...)`, not a Window-menu-only branch. When the dispatcher reports a new scroll offset, `handle_scroll(...)` stores it, rebuilds the surface, and dispatches a synthetic move at the same pointer position. That second dispatch recomputes `hovered_item_index` from the current scroll offset and prevents hover/action rows from lagging behind the visible popup position.

For multi-column popup layouts, the scroll extent is the height of the tallest column rather than the total number of menu rows. This keeps scroll state aligned with the grid rows shown inside the popup viewport.

Overwide menu bars use `menu_bar_content_width` plus `menu_bar_scroll_offset`. The pointer surface clips menu buttons to the visible top-bar viewport, scrolls horizontally when the menu bar is wider than the shell, and opens extension menu buttons after they are scrolled into view. The host state publishes that same offset as `HostMenuStateData.menu_bar_scroll_px`, so Rust-owned native text/border painting, popup anchoring, and native pointer guards all move with the same geometry.

## Native Chrome Projection

Workbench chrome projection now keeps menu items as a tree through `HostMenuChromeItemData.children`. Root popup rows come from `workbench_menu_popup.zui`, while native child popups are painted from the same tree and `HostMenuStateData.open_submenu_path`. The menu-bar v2 stencil still owns the top-row shape, and `chrome_template_projection/menu_chrome.rs` clones that stencil horizontally for extension menus beyond slot 6 so projected `menu_frames` and shared pointer frames do not truncate plugin top-level menus. Each cloned slot is remeasured from the live label before x advancement, which removes the old `label.chars().count() * 7 + 24` drift. The native pointer guard also treats open child popup frames as menu-owned space, so pointer events over a child popup stay in the shared menu pointer route instead of falling through to panes or the viewport.

## Test Coverage

`shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell` verifies the popup frame and item hit nodes stay inside a shell that is narrower than one nominal popup column and shorter than `POPUP_MIN_HEIGHT`.

`shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp` opens a right-edge menu in `MenuOverflowMode::MultiColumn`, forces the popup to clamp horizontally inside a small shell, clicks the first row of the second column, and verifies the route remains absolute item index `9` with action id `Overflow.Action09`.

`shared_menu_pointer_layout_uses_chrome_menu_overflow_preference` verifies the production layout builder consumes `EditorChromeSnapshot.menu_overflow_mode` instead of hard-coding `Auto`.

Existing menu pointer coverage still owns the single-column paths: opening and dismissing top-level menus, resolving popup item routes, scrolling Window popups through shared scroll state, recomputing hover after scroll, and dispatching operation-backed leaves.

`shared_menu_pointer_bridge_opens_flipped_nested_popup_for_branch_hover` verifies that hovering a branch opens a child popup, flips it left near the shell edge, and routes the nested leaf using the preserved pre-order item index.

`shared_menu_pointer_layout_extends_menu_button_frames_for_extension_menus` verifies production menu pointer layout derives slots beyond the authored seven-slot menu bar stencil and exposes scrollable content width for overwide extension menus.

`shared_menu_pointer_layout_content_measures_root_popup_widths` verifies a long extension label plus `Ctrl+Shift+Alt+U` expands its root popup beyond the old 208-pixel fallback while remaining clamped to the shell. `shared_menu_pointer_bridge_recomputes_hovered_item_after_window_popup_scroll` now derives its probe x from the projected Window button frame; this keeps the regression aligned with runtime-measured top-level menu widths instead of relying on the old hard-coded x coordinate.

`shared_menu_pointer_bridge_scrolls_overwide_menu_bar_to_extension_button` verifies the shared pointer surface scrolls an overwide top menu bar horizontally, clips button hit frames to the viewport, and opens an extension menu button after it is brought into view.

`shared_menu_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state` verifies identical layout/state syncs do not rebuild the bridge-local pointer surface, while all scroll/hover/open-menu tests above continue to prove real state changes rebuild the route surface.

`menu_chrome_nodes_project_extension_slots_beyond_authored_stencil` verifies native chrome projection clones menu slot nodes beyond `MenuSlot6` while preserving the extension label and monotonically increasing x position.

`menu_chrome_nodes_measure_top_level_slots_with_runtime_font_width` and `shared_menu_pointer_layout_measures_menu_labels_with_runtime_font_width` verify visual menu slots and shared pointer hitboxes both use runtime glyph widths. The pair locks the same-character-count case where `iiiiiiii` and `WWWWWWWW` must not receive the same slot width.

`shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model` verifies a nested `EditorOperationPath` leaf dispatches through the shared pointer route into the operation runtime.

`rust_owned_host_painter_draws_open_nested_menu_popup` verifies the Rust-owned native host painter produces visible child popup pixels from `HostMenuChromeItemData.children` and `HostMenuStateData.open_submenu_path`.

`capture_nested_menu_popup_visual_artifact` writes `target/visual-layout/editor-window-20260507-nested-menu-popup-900x620.png` so the visible native-host result can be inspected alongside the focused route and painter tests.
