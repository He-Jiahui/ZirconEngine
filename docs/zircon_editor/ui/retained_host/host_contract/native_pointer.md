---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
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
  - zircon_editor/src/tests/host/retained_window/shell_window.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
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
  - cargo test -p zircon_editor --lib native_host_menu_click_unions_text_focus_damage_without_full_frame --offline --message-format=short
  - cargo test -p zircon_editor --lib redraw_region_can_request_frame_update_without_losing_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_viewport_toolbar_only_dispatches_primary_press --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_late_viewport_toolbar_controls --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_resize_splitter_forwards_move_and_release_after_capture --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_releases_capture_and_forwards_drop --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_cross_dock_release_uses_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_document_edge_release_uses_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_floating_window_release_uses_floating_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_close_prompt_button_press_uses_overlay_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_document_tab_with_document_region_origin --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_floating_document_tab_press_uses_floating_window_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_floating_window_header_press_uses_floating_layer_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_drawer_header_tab_press_uses_drawer_region_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_activity_rail_press_uses_center_band_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_host_page_tabs_with_tab_local_point --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_hierarchy_press_uses_pane_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_hierarchy_move_prefers_native_hover_when_template_node_overlaps --locked --jobs 1 --message-format=short (2026-05-15: passed)
  - cargo test -p zircon_editor --lib native_host_template_node_move_updates_hover_without_rebuilding_presentation --locked --jobs 1 --message-format=short (2026-05-15: passed)
  - cargo test -p zircon_editor --lib native_host_hierarchy_move --locked --jobs 1 --message-format=short (2026-05-15: passed, 2 passed)
  - tools/ui-profile-capture.ps1 -Scenario idle_hover -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260515-211644-idle_hover: redraw_region_count=1, gpu_draw_calls=10, gpu_visible_draw_items=12, zero alerts/fallback)
  - cargo test -p zircon_editor --lib frame_update_region_queues_external_redraw_with_frame_update --locked --jobs 1 --message-format=short (2026-05-16: passed)
  - cargo test -p zircon_editor --lib close_requested_callback_can_mutate_host_state_without_reentrant_borrow --locked --jobs 1 --message-format=short (2026-05-16: passed)
  - tools/ui-profile-capture.ps1 -Scenario viewport_image -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260516-000208-viewport_image: dirty_paint_only_count=1, redraw_region_count=1, gpu_draw_calls=16, gpu_visible_draw_items=21, zero alerts/fallback)
  - cargo test -p zircon_editor --lib retained_window --offline --message-format=short
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

## Menu Damage

Menu hit testing and popup damage math are isolated in `native_pointer/menu_geometry.rs`. `native_pointer.rs` still owns pointer routing and callback dispatch, but the menu helper module owns the top-bar, popup, nested-popup, and menu-damage rectangles.

Primary menu clicks measure menu damage before and after `HostMenuStateData` changes, then request a regional repaint for the union of those rectangles. If the same click clears a focused text input, the old edit frame is unioned into that region as well. This keeps menu open/close and focus-outline cleanup on the paint-only path instead of requesting a full host frame update.

## Viewport Toolbar Damage

Viewport toolbar hit routing takes priority over the viewport body, but it no longer treats every primary press as a full host-frame repaint. Common mode and toggle controls dispatch their Rust callback, request a presentation frame update, and preserve the toolbar rectangle as the damage region. That lets the presenter repaint the changed toolbar labels or selected states without repainting the full editor window.

Play-mode, frame-selection, and view-alignment controls can affect session state, status text, and viewport camera state. `native_pointer/viewport_toolbar_damage.rs` therefore gives them the conservative center-band plus status-bar damage region: the callback can still patch presentation state before repaint, but menu/title chrome no longer gets redrawn.

## Viewport Frame Update Retry

Viewport rendering can need more than one retained-host tick during startup. The first tick may only start the lazy render-framework resolver; no viewport exists yet, so `submit_extract_with_ui` returns without a frame. `RetainedEditorHost::tick()` now keeps `render_dirty` in that case and queues a frame-update redraw for the committed viewport content region through `UiHostWindow::request_frame_update_region`.

That redraw is queued as an external request and drained by the native event loop, so it does not call the frame callback while `RetainedEditorHost` is already mutably borrowed. When the resolver finishes, the next redraw invokes the normal frame update, submits the extract, polls the viewport image, and then the existing paint-only viewport-image patch repaints only the changed viewport rectangle. If the committed viewport rectangle is not visible yet, the queued request falls back to a full frame-update redraw so startup cannot stall on a zero-sized bootstrap frame.

## Pane Button Damage

Generic pane button presses now use `native_pointer/pane_button_damage.rs`. The dispatcher still calls the pane-specific callback first, then requests a frame update with the pane body, center band, and status bar as damage. This keeps selection and status changes visible without repainting the entire native window.

Pure release remains paint-local to the pane frame, and inert panes without a callback-visible change can still return idle. The regional press path is intentionally broader than the row/control hit because pane callbacks can update sibling pane chrome or status text.

## Close Prompt Damage

Close prompt action buttons are routed before pane and chrome hit testing. `native_pointer/close_prompt_damage.rs` returns the overlay/dialog union and the dispatcher keeps the required frame update attached to that bounded region. This removes the old unconditional full-frame request from save/discard/cancel button presses while still covering the modal overlay disappearing or changing state.

## Chrome Press Damage

Top-level chrome press dispatch no longer treats every successful click as a full native frame repaint. `native_pointer/chrome_damage.rs` owns the damage mapping for controls whose effect stays inside a known region: root document-tab press and close use the document dock, floating document-tab press and close use the owning floating-window frame, drawer-header tab press uses the corresponding drawer dock, and activity-rail press uses the center band because opening or swapping a drawer can move the document region.

Floating-window header focus uses the union of all floating-window frames. The previous focused floating window is tracked in the retained app layer, not in the native host contract presentation packet, and focusing can reorder floating windows. The floating-layer union is therefore the conservative regional damage for z-order/focus changes: it avoids repainting menu, document, drawer, and status regions while still covering stacked floating-window pixels that can change.

The dispatcher still requests a frame update for these paths, so Rust callbacks can patch presentation state before repaint. It preserves the region damage so the presenter avoids redrawing menu chrome, title chrome, and other unaffected pixels. Host-page tabs now use a conservative multi-region union: page tab row, project path, page chrome template nodes, center band, and status bar. This covers page activation and status changes while keeping the menu/title chrome out of the repaint.

## Resize Capture

Resize splitters live in `HostResizeLayerData` and are hit by `route_top_level_chrome(...)` as `ChromePointerRoute::Resize`. A splitter press starts a host resize session by storing `HostResizeStateData` in `HostContractState` and emitting `HOST_POINTER_DOWN`.

While that resize state is active, root pointer move dispatch short-circuits before tab dragging, menus, and pane routing. Every move emits `HOST_POINTER_MOVE`, updates the stored pointer coordinates, and requests a frame update with regional damage. Primary release emits `HOST_POINTER_UP`, clears the resize state, and uses the same regional damage path.

This is a capture-style contract: after the user presses the split bar, later move/up events do not need to remain physically over the splitter hitbox. The layout resize implementation in the retained host app can therefore receive the complete down/move/up sequence needed to adjust drawer and workspace bounds.

The resize damage rectangle is the committed center band, computed in `native_pointer/resize_damage.rs`. Layout changes can move drawer, document, splitter, and viewport frames, so the damage is intentionally broader than the splitter itself; it still avoids repainting menu chrome and status bar pixels that resize capture does not mutate.

## Tab Drag Release Damage

Tab drag release still emits `HOST_POINTER_UP` before clearing capture, but it no longer blindly requests a full native frame for every active drag. After the callback updates `HostDragStateData.active_drag_target_group`, the dispatcher computes release damage from `native_pointer/tab_drag_damage.rs`.

Resolvable dock releases use this regional path. When `drag_source_group` and `active_drag_target_group` are the same local dock group, the redraw request keeps the required frame update and limits damage to that dock plus status. When both groups are known but different local docks, or when the target is a `document-left/right/top/bottom` edge split, the damage widens to center band plus status because the move can rearrange drawers and document chrome.

Floating-window target groups are matched against the `FloatingWindowData` target-group fields in the committed presentation. A drop onto an existing floating window repaints that floating frame and also widens to center/status when the source was a local dock. Detach/drop-empty targets remain full-frame because the newly created floating-window bounds are not present in the old presentation packet used by native pointer dispatch.

## Pointer Rebuild Guard

Retained host pointer callbacks now call `RetainedEditorHost::use_committed_pointer_layout()` instead of `recompute_if_dirty()`. Pointer routing uses the last committed bridge frames, while presentation/layout/window dirty flags remain queued for `tick()` or explicit `refresh_ui()`. This prevents native pointer down/move/up/scroll callbacks from recursively rebuilding the full editor UI tree during hit routing.

The boundary test `pointer_handlers_do_not_force_slow_path_rebuilds` scans the pointer modules for this contract. It covers viewport, viewport toolbar, menu, workbench tab/header, asset tree/content/reference, hierarchy, detail scrollers, welcome recent-project pointers, and drag/resize docking pointer paths.

Bridge-local pointer surfaces also avoid redundant rebuilds. Activity rail, document tabs, drawer headers, host page tabs, viewport toolbar, and viewport overlay sync paths compare the incoming layout with the committed layout and return `false` without rebuilding when the layout is unchanged. Menu, hierarchy, detail scroll surfaces, and asset tree/content/reference lists use the stricter `layout + state` equality check, so hover, scroll, popup, and open-submenu state changes still rebuild while identical projection packets are ignored. The welcome recent list reads recent-project paths from `HostPresentationCache`; click syncs the cached paths and high-frequency move/scroll callbacks use a size-only sync that preserves them instead of rebuilding `EditorChromeSnapshot` on every hover event. This keeps high-frequency pointer routing from paying a fresh surface build when the projection pass re-sends identical frames, while still rebuilding immediately after real geometry, candidate, or state changes.

## Template Hover Damage

Template-backed pane bodies can now update visible hover state without rebuilding the retained presentation. Pointer move dispatch stores the active template control id and frame in `HostPaneInteractionStateData`, then `window.rs` overlays that hover bit when publishing the next host presentation. The base presentation cache remains unchanged, so repeated hover frames do not increment `presentation_rebuild_count`.

`native_pointer/template_hover_damage.rs` computes the union of the old and new template hover frames. Pointer moves that enter, leave, or switch template nodes request only that regional repaint, while repeated moves over the same node return idle. When the move also touches a native-hover pane such as Hierarchy, Welcome, Assets, or AssetBrowser, the move-specific pane router prefers the native hover target over overlapping `TemplateNode` hits, preserving the existing row-hover callbacks and damage semantics for those panes.

The profile closeout `20260515-211644-idle_hover` passed the automated interaction gate with one regional redraw, `gpu_draw_calls=10`, `gpu_visible_draw_items=12`, zero hotspot alerts, and no software fallback. This gives `idle_hover` real GPU patch evidence rather than only pointer-frame samples.

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
