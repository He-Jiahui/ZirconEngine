---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/detail_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/tree/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract.rs
  - zircon_editor/src/tests/host/retained_activity_rail_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_document_tab_pointer/dispatch.rs
  - zircon_editor/src/tests/host/retained_drawer_header_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_host_page_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/pointer_bridge.rs
  - zircon_editor/src/tests/host/retained_detail_pointer/scroll_bridge.rs
  - zircon_editor/src/tests/host/retained_list_pointer/bridge_dispatch.rs
  - zircon_editor/src/tests/host/retained_asset_pointer.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/workbench/layout/drawer_attachment.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/detail_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/tree/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_sync.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
plan_sources:
  - user: 2026-05-07 editor input, split bar drag, drawer drag selection, and single-select regressions
tests:
  - zircon_editor/src/tests/host/retained_window/native_host_contract.rs
  - zircon_editor/src/tests/workbench/layout/drawer_attachment.rs
  - cargo test -p zircon_editor template_assets -- --nocapture (2026-05-11: passed, 10 passed)
  - cargo test -p zircon_editor pointer_handlers_do_not_force_slow_path_rebuilds -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor native_host_viewport_button_and_scroll_wait_for_viewport_image_repaint -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib shared_activity_rail_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_document_tab_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_drawer_header_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_host_page_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_viewport_toolbar_pointer_bridge_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib viewport_overlay_pointer_router_skips_rebuild_for_unchanged_layout -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_menu_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_scroll_surface_bridge_skips_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_welcome_recent_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_hierarchy_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib shared_asset_pointer_bridges_skip_rebuild_for_unchanged_layout_and_state -- --nocapture (2026-05-11)
  - cargo test -p zircon_editor --lib retained_activity_rail_pointer -- --nocapture (2026-05-11: passed, 7 passed)
  - cargo test -p zircon_editor --lib retained_document_tab_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor --lib retained_drawer_header_pointer -- --nocapture (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_editor --lib retained_host_page_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor --lib retained_viewport_toolbar_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor --lib viewport_overlay_pointer_router -- --nocapture (2026-05-11: passed, 4 passed)
  - cargo test -p zircon_editor --lib retained_menu_pointer -- --nocapture (2026-05-11: passed, 22 passed, 4 ignored)
  - cargo test -p zircon_editor --lib retained_detail_pointer -- --nocapture (2026-05-11: passed, 7 passed)
  - cargo test -p zircon_editor --lib retained_list_pointer -- --nocapture (2026-05-11: passed, 7 passed)
  - cargo test -p zircon_editor --lib retained_asset_pointer -- --nocapture (2026-05-11: passed, 7 passed)
  - cargo check -p zircon_editor --lib --tests --locked --message-format=short
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir target/codex-shared-a
doc_type: module-detail
---

# Native Pointer Host Contract

The native retained host contract is the editor-side boundary that translates root window pointer and keyboard input into editor shell callbacks. `native_pointer.rs` owns hit routing for top-level chrome, pane surfaces, template nodes, tab dragging, and shell resize splitters. `window.rs` owns keyboard/text dispatch after `native_pointer.rs` has established a focused text-input target.

## Text Input Focus

Template text focus is intentionally keyed by an edit target, not only by the visual control id. Existing showcase controls provide `edit_action_id` directly, while generic template surfaces often provide only a binding id for `UiEventKind::Change`.

The focus target resolution order is:

1. Use `TemplateNodePointerHit.edit_action_id` when projection already supplied one.
2. Use the welcome text action id for the legacy welcome text route.
3. For generic `input-field` and `number-field` nodes, fall back to `binding_id` when there is no explicit edit action.

`pane_component_projection::host_template_node(...)` now derives generic template edit and commit targets from bindings. `UiEventKind::Change` becomes the edit target, and `UiEventKind::Submit` becomes the commit target. Showcase-specific preferred action discovery still wins first, so existing curated showcase controls keep their explicit action mapping.

This keeps binding-only template inputs typable: pointer press can focus the node, and `window.rs` can dispatch typed text through `on_surface_control_edited(control_id, edit_target_id, new_value)` instead of dropping the key event because the focus target was empty.

## Resize Capture

Resize splitters live in `HostResizeLayerData` and are hit by `route_top_level_chrome(...)` as `ChromePointerRoute::Resize`. A splitter press starts a host resize session by storing `HostResizeStateData` in `HostContractState` and emitting `HOST_POINTER_DOWN`.

While that resize state is active, root pointer move dispatch short-circuits before tab dragging, menus, and pane routing. Every move emits `HOST_POINTER_MOVE`, updates the stored pointer coordinates, and requests a full-frame redraw. Primary release emits `HOST_POINTER_UP`, clears the resize state, and requests another full-frame redraw.

This is a capture-style contract: after the user presses the split bar, later move/up events do not need to remain physically over the splitter hitbox. The layout resize implementation in the retained host app can therefore receive the complete down/move/up sequence needed to adjust drawer and workspace bounds.

## Pointer Rebuild Guard

Retained host pointer callbacks now call `RetainedEditorHost::use_committed_pointer_layout()` instead of `recompute_if_dirty()`. Pointer routing uses the last committed bridge frames, while presentation/layout/window dirty flags remain queued for `tick()` or explicit `refresh_ui()`. This prevents native pointer down/move/up/scroll callbacks from recursively rebuilding the full editor UI tree during hit routing.

The boundary test `pointer_handlers_do_not_force_slow_path_rebuilds` scans the pointer modules for this contract. It covers viewport, viewport toolbar, menu, workbench tab/header, asset tree/content/reference, hierarchy, detail scrollers, welcome recent-project pointers, and drag/resize docking pointer paths.

Bridge-local pointer surfaces also avoid redundant rebuilds. Activity rail, document tabs, drawer headers, host page tabs, viewport toolbar, and viewport overlay sync paths compare the incoming layout with the committed layout and return `false` without rebuilding when the layout is unchanged. Menu, hierarchy, welcome recent list, detail scroll surfaces, and asset tree/content/reference lists use the stricter `layout + state` equality check, so hover, scroll, popup, and open-submenu state changes still rebuild while identical projection packets are ignored. This keeps high-frequency pointer routing from paying a fresh surface build when the projection pass re-sends identical frames, while still rebuilding immediately after real geometry, candidate, or state changes.

## Drawer Active Selection Normalization

Drawer headers and tab dragging can expose stale layout states when `drawer.tab_stack.active_tab` and `drawer.active_view` diverge. `LayoutManager::apply(...)` now normalizes all activity-window drawers after every changed layout command and before syncing legacy drawer mirrors.

For each drawer, the normalization rule is:

1. If the drawer is collapsed or has no tabs, both `active_tab` and `active_view` become `None`.
2. Otherwise, keep the existing valid `active_tab` when possible.
3. If `active_tab` is missing or stale, fall back to a valid `active_view`.
4. Write the same resolved id back to both fields.

The resulting invariant is zero or one active drawer item per drawer. This prevents a selected drawer title drag/drop path from leaving one active id in the tab stack and another in the drawer view field, which looked like multi-selection at the target location.

## Focused Regression Coverage

`native_host_binding_only_template_text_field_accepts_keyboard_input` covers a template input that has `component_role = "input-field"` and `binding_id`, but no explicit edit action id. It focuses the node and expects a typed character to route through `on_surface_control_edited`.

`native_host_resize_splitter_forwards_move_and_release_after_capture` covers splitter press, move, and release. The expected callback sequence is down, move, up with the root pointer positions forwarded to host resize callbacks.

`drawer_selection_is_normalized_to_one_active_item_after_layout_commands` creates a drawer state with conflicting `active_tab` and `active_view`, applies a layout command, and verifies both fields are normalized to the same valid item.

The shared pointer bridge rebuild guards have focused tests for unchanged activity rail, document tab, drawer header, host page, viewport toolbar, viewport overlay, menu, detail scroll, welcome recent, hierarchy, and asset-list inputs. The drawer header dispatch regression now derives its expected `left_top` instance from the projected pointer layout because current editor presets can place Hierarchy before Project in that runtime harness, while the routing contract is the projected tab, not a fixed legacy instance id.

The 2026-05-09 retained-host validation rerun exposed two host-scoped test-build residues after the Slint build dependency was removed. Retained host tests still called `i_retained_backend_testing::init_no_event_loop()`, and one hierarchy projection test still built `ModelRc` from a borrowed slice. Both were deleted or converted to the Rust-owned `model_rc(Vec<T>)` path because `UiHostWindow::new()` now initializes pure Rust host-contract state and no backend bootstrap crate is part of the retained contract.

The focused editor test-build check `cargo check -p zircon_editor --lib --tests --locked --message-format=short` now compiles with warnings only. The workspace validator gets past `cargo build --workspace --locked` and fails later in `cargo test --workspace --locked` only if a new active-worktree blocker appears outside this native pointer contract; do not restore a backend-testing crate to hide stale test calls.
