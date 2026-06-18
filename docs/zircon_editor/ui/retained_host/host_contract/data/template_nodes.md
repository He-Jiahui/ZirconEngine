---
related_code:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/app/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/context_menu.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_output_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session_control_tests.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/popup_frame.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs
  - zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
implementation_files:
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/app/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/context_menu.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_projection.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/panel_output_tests.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session_control_tests.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/popup_frame.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
plan_sources:
  - user: 2026-05-15 continue Zircon Editor Demo first-screen and .zui showcase plan
  - user: 2026-05-20 migrate Slint Material component behavior into retained Editor UI without direct Slint runtime
  - .codex/plans/Zircon Editor Demo 首屏与 .zui 组件陈列计划.md
  - docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md
tests:
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - build_export_wizard_panel_nodes_project_retained_export_wizard_panel
  - build_export_wizard_surface_action_maps_panel_buttons_to_session_actions
  - desktop_export_wizard_sessions_project_view_model_after_generate_plan
  - export_wizard_panel_session_start_updates_controls_before_worker_poll
  - export_wizard_panel_session_cancel_disables_cancel_before_terminal_poll
  - export_wizard_panel_template_state_projects_stage_stdout_and_stderr
  - export_wizard_panel_route_prefers_action_over_binding
  - zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/context_menu.rs
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer.rs inline unit tests
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs zircon_editor/src/ui/retained_host/host_contract/window.rs zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs (2026-06-14: passed after native popup text-search keyboard baseline)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs zircon_editor/src/ui/retained_host/host_contract/window.rs zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs (2026-06-14: passed after native popup Home/End keyboard boundary baseline)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs (2026-06-14: passed after native PopupRow projection parity baseline)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/layouts/views/view_projection.rs zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/popup_frame.rs zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs (2026-06-14: passed after M3.S1 popup shell role/projected frame geometry baseline)
  - node docs/ui-and-layout/ai-workbench-style/component-prototype/verify-native-component-contract.mjs (2026-06-15: passed after M5.S1 native popup keyboard row-geometry contract alignment)
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs (2026-06-15: passed after M5.S1 native popup keyboard row-geometry contract alignment)
  - rustfmt --edition 2021 touched DragOverlay Workbench/native Rust files (2026-06-15: passed after M3.S4 DragOverlay Workbench/native parity baseline)
  - cargo test -p zircon_editor --lib apply_presentation_carries_componentized_workbench_window_nodes_separately --locked -- --nocapture --test-threads=1
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/drag_overlay_tests.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_drag_overlay.rs
  - cargo test -p zircon_editor --lib component_showcase_template_nodes_preserve_scroll_clip_frames --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 1 passed)
  - cargo test -p zircon_editor --lib component_showcase --locked --target-dir target/codex-shared-b --message-format short -- --test-threads=1 (2026-05-15: passed, 24 passed)
  - cargo test -p zircon_editor --lib template_nodes --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 7 passed)
  - cargo test -p zircon_editor --lib dual_host_parity_preserves_layout_attributes_and_routes_for_representative_documents --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed, 1 passed)
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b --message-format short (2026-05-15: passed)
  - cargo check -p zircon_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 BuildExport wizard_view_model handoff: passed with existing warnings after CommandPalette row type recovery)
  - cargo test -p zircon_editor --lib build_export_wizard_session --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never (2026-06-15 BuildExport wizard_view_model handoff focused tests: timed out after 904 seconds without target output; matching cargo/rustc leftovers stopped)
  - cargo test -p zircon_editor --lib export_wizard_panel_session_start_updates_controls_before_worker_poll --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 Start/Cancel control focused test: blocked before target test by unrelated RenderQualityProfile::with_history_resolve and notification_center.rs partial-move compile drift)
  - cargo test -p zircon_editor --lib export_wizard_panel_template_state_projects_stage_stdout_and_stderr --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-editor-dispatch-0614 --message-format short --color never -- --exact --nocapture (2026-06-15 stage stdout/stderr projection focused test: blocked before target test by unrelated RenderQualityProfile::with_history_resolve compile drift)
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (2026-05-15: passed)
  - E:\zircon-build\ZirconEngine\zircon_editor.exe --list-operations --headless (2026-05-15: passed, includes window.ui_component_showcase.open)
  - .codex/run-logs/editor-noargs-smoke-polished.png (2026-05-15: no-argument editor smoke screenshot, Component Showcase first screen visible without bottom-log overlap)
doc_type: module-detail
---

# Template Pane Nodes

`TemplatePaneNodeData` is the retained host DTO for editor-authored template panes. It carries the visual and interaction facts that the native host needs after a shared `UiSurface` has already compiled and arranged the source document. The host contract must not rederive template geometry from component names or from per-pane coordinate tables.

## Action And Binding Identity

`action_id` is the executable route identity. Projection chooses the first authored route action for click, toggle, change, commit, or edit behavior, falling back to a normalized binding id only when the route has no explicit action. Native activation and host dispatch must treat this field as the command/action path to execute.

`binding_id` is the authored template binding identity. It preserves source metadata such as `ComponentLab/Primary` for diagnostics, round-trip checks, and legacy lookup compatibility, but it is not the preferred executable id when an explicit runtime action such as `component_lab.button.primary` exists.

Workbench window node projection follows the same split as pane node projection. `workbench_window_projection.rs` emits `action_id` for dispatch and keeps the authored binding in `binding_id`, so host-contract tests should assert both fields instead of requiring action ids to carry template binding paths.

## Spatial Authority

`frame` and `clip_frame` are pane-local layout facts projected from the shared runtime surface. `EditorUiHostRuntime::build_host_model_with_surface(...)` prefers `UiArrangedNode.frame` and `UiArrangedNode.clip_frame` from `surface.arranged_tree`; tests and metadata-only paths that pass an uncomputed surface still fall back to the raw `UiTreeNode.layout_cache`. On the normal laid-out pane path, the arranged clip frame is the effective clip after intersecting all clipping ancestors, so a descendant inside a `ScrollableBox` carries the scroll viewport clip even when its own frame extends far below the visible pane.

`has_clip_frame` distinguishes a shared-surface node with an arranged clip from older projection-only synthetic nodes. `pane_component_projection::host_template_node(...)` serializes the optional host-model clip into `has_clip_frame` plus `TemplateNodeFrameData`, while `template_node_conversion.rs` keeps projection-only fallbacks at `has_clip_frame = false`.

`pane_data_conversion/build_export_wizard_panel.rs` is a scoped adapter for the
desktop export plugin panel. It consumes the host-owned
`RetainedUiHostProjection` produced from the private
`editor_build_export_desktop.panel` template and maps it into the existing
`TemplatePaneNodeData` contract, including frame, clip, disabled state,
validation/message fields, text/value strings, option/list strings, and primary
action/binding ids. The surrounding BuildExport pane conversion only uses this
adapter for real `BuildExportV1` pane presentations and keeps the older target
row nodes as fallback when the plugin panel projection is unavailable.
For nodes with `dispatch_kind = "export_wizard_panel"`, primary activation uses
`action_id` before `binding_id` so the panel can preserve authored
`DesktopExportWizard/...` binding metadata while routing clicks to host-owned
`workbench.build_export.*.<profile>` actions.
`BuildExportPaneViewData.wizard_view_model` is the live session handoff for
this path. When the retained app has a profile-keyed
`ExportWizardPanelSession`, the pane adapter consumes that view model so button
enabled state, stage rows, diagnostics, and terminal cleanup reflect the host
session. Without a session, the adapter still builds the earlier synthetic
dry-run view model from the target rows so the panel keeps a stable first render.
The session/view-model boundary now projects Start and Cancel state on the same
frame as the request: Start marks the model active before the worker's first
event is polled, Cancel marks the snapshot Cancelling before the terminal event
returns, and worker join clears the active state. `TemplatePaneNodeData.disabled`
therefore follows the host session immediately instead of waiting for the next
background event drain.
The export wizard TerminalOutput slot now also receives per-stage stdout/stderr
from the host-owned execution result. Those lines keep stable
`stage-output.<stage>.<stream>.<index>` keys before they are adapted into
synthetic `TemplatePaneNodeData` rows, so the native pane can display captured
CLI output without inspecting job execution internals.

## Painting

`host_contract/paint_template_nodes/template_nodes.rs` intersects three regions before emitting paint commands:

- the pane body clip;
- the active frame damage clip, when the native presenter is repainting a subregion;
- the node's own retained clip frame, when `has_clip_frame` is true.

This matters for image and text nodes because image rasterization can be expensive and text can otherwise bleed outside a scroll viewport. The painter skips nodes whose frame does not intersect the effective clip before requesting image pixels, then passes the same clip into quad, image, and text commands.

## Slint Material State Metadata

The retained Material migration carries Slint-derived state-layer and ripple facts directly on `TemplatePaneNodeData` so the native host can paint Material behavior without linking Slint or generated `.slint` code.

The state-layer/ripple fields are:

- `enter_pressed`: keyboard activation metadata equivalent to `FocusTouchArea.enter_pressed`; runtime input routing owns when it is set.
- `state_layer_enabled`: opt-in for drawing a Material state-layer overlay.
- `state_layer_color`: source `StateLayerArea.color` / `Ripple.color`; the painter falls back to the host focus-ring color when it is transparent.
- `ripple_enabled`: opt-in for the retained static press ripple.
- `ripple_pressed_x` and `ripple_pressed_y`: source-compatible press origin metadata from `pressed_x` / `pressed_y` or explicit retained ripple attrs.
- `ripple_unclipped`: host-internal inverse of source `clip_ripple`.

Projection accepts both retained names and source-compatible names. `pane_component_projection::host_template_node(...)` maps `state_layer_enabled` / `display_state_layer`, `state_layer_color` / `ripple_color` / `color`, `ripple_enabled` / `ripple`, `ripple_pressed_x` / `pressed_x`, `ripple_pressed_y` / `pressed_y`, and `clip_ripple`. Older conversion paths in `template_node_conversion.rs` default all M2 metadata to inactive values so existing template nodes do not accidentally draw overlays.

`host_contract/paint_template_nodes/material_state_layer.rs` applies the retained priority `disabled > focus/selected/checked > pressed/enter_pressed > drag > hover/drop/active-drag-target > default`. Disabled uses focus opacity to preserve the source `root.state_layer_opacity: MaterialPalette.state_layer_opacity_focus` behavior for disabled display backgrounds. `ripple_enabled` does not imply `state_layer_enabled`, so callers may request only the static press ripple without the full overlay. Ripple is intentionally static in M2: it draws a width-derived press-origin circle with press opacity while animation timing remains metadata in `editor_material.v2.ui.toml` for a later motion layer.

## Structured Popup Rows

`TemplatePaneOptionData` and `TemplatePaneMenuItemData` are the retained host row DTOs for popup option and menu content. They carry identity, display label, selection/check state, disabled state, focus/hover/press state, matching state, and now `loading` so the native `template_popup_rows.rs` painter can drive `WorkbenchPopupRowState` without rereading source `.zui` metadata. `TemplatePaneOptionData` also carries optional row description, tone, and unread state for component-owned overlay painters such as `NotificationCenter`; popup-row painters ignore those fields while `template_notification_center.rs` uses them for body text, severity marker color, and unread row emphasis.

`pane_option_projection.rs` treats real `DropdownPopup` option metadata as a structured source instead of a flat string list. It accepts `id|label=...` or `id|text=...` row declarations, merges `value`, `selected_options`, and MUI-style `selectedOptions`, and projects `disabled_options`, `focused_options`, `hovered_options`, `pressed_options`, `loading_options`, `focused_index`, and `hovered_option_id` into per-row state. Matching compares option id, label, and raw declaration so existing authored values and new canonical ids both work.

`pane_menu_projection.rs` keeps legacy menu item parsing but now preserves the `loading` flag in `TemplatePaneMenuItemData`. This keeps `ContextActionMenu` and real `ContextMenu` row metadata aligned with the same native popup-row selector priority used by runtime render extract.

`WorkbenchContextMenuRequestData` is the retained host DTO for editor Workbench right-click routing. It carries the hit target control id, authored action/dispatch/role metadata, display text, semantic target path, pointer anchor, and provider-generated menu rows. `host_contract/workbench_context_menu.rs` creates this request from `TemplateNodePointerHit` on secondary press: scene rows map to scene menu actions, module rows map to module actions, actionable Workbench controls fall back to inspect/copy/reveal actions, and existing popup rows are ignored to prevent nested context menus. The host contract does not execute the selected action here; it only asks the retained app to open the Workbench context-menu overlay with enough target metadata for the later command route.

`pane_component_projection::popup_frame.rs` now recognizes `context-menu`, `context-action-menu`, `dropdown-popup`, and `notification-center` as anchor-positioned overlay roles. That means `TemplatePaneNodeData.frame` is already the projected popup frame when these shell nodes carry `popup_anchor_x`/`popup_anchor_y`; native painter, hit-test, keyboard navigation, and outside-dismiss code must consume that frame as popup geometry rather than re-deriving another trigger-relative frame.

`pane_component_projection/notification_center.rs` maps `NotificationCenter.notifications` into the same structured option model for native painting. Entries may be pipe strings or TOML tables; id/title/message/tone/unread/disabled values are preserved, `selected_notification_id` marks the selected row, `focused_index` marks the focused row, and `visible_limit` clamps the projected row count before the native painter receives it.

## DragOverlay Metadata

`TemplatePaneNodeData` now carries DragOverlay-specific native paint metadata instead of overloading generic layout fields. The projected fields are `drag_payload_kind`, `drag_payload_label`, `drag_payload_reference`, `has_drag_cursor`, `drag_cursor_x`, `drag_cursor_y`, `drag_offset_x`, `drag_offset_y`, `drag_preview_width`, `drag_preview_height`, `drop_allowed`, `has_drop_target`, `drop_target_x`, `drop_target_y`, `drop_target_width`, `drop_target_height`, `drop_indicator_edge`, and `drop_indicator_text`.

`pane_component_projection/drag_overlay.rs` is the only retained-host projector for these fields. It treats `open = true` or `dragging = true` as `popup_open`, keeps payload label/reference in both dedicated drag fields and the pane's text/value text lanes for inspection, records cursor-relative preview geometry, and preserves allowed/blocked drop target metadata for the native painter.

`template_drag_overlay.rs` consumes these fields as a component-owned overlay. Closed overlays return handled without drawing, so generic text/surface fallback cannot leak a stale drag label. Open overlays draw the preview chip from cursor/offset/preview size metadata and draw the drop indicator from the explicit drop target rectangle and `drop_indicator_edge`.

`template_popup_layout.rs` owns that distinction for option rows. Ordinary Dropdown/ComboBox rows still derive a bounded popup below or above the trigger frame, while standalone `DropdownPopup` rows are cut directly inside the projected popup frame. The layout module exposes `template_option_rows_use_projected_frame(...)` for host-contract consumers; `native_keyboard.rs` uses that predicate to route ordinary dropdown keyboard hover frames through `dropdown_option_row_frame_within` and projected `DropdownPopup` frames through `template_option_row_frame_within`. `template_popup_rows.rs`, `surface_hit_test/template_node.rs`, `native_keyboard.rs`, and `native_popup_dismiss.rs` therefore keep visible rows, pointer targets, keyboard focus frames, and dismiss damage aligned.

`native_keyboard.rs` treats ArrowUp/ArrowDown as roving popup navigation, Home/End as boundary navigation, and single-event text input as first matching enabled-row search over the active popup shell. Option popups dispatch `workbench_option` hover identity through the option id, while menu popups dispatch `workbench_menu_item` hover identity through the action id, so first/last jumps and character search update the same retained hover fields that pointer movement and arrow navigation already use.

## Hit Testing

`host_contract/surface_hit_test/template_node.rs` reconstructs a temporary surface for template-node hit testing. When a `TemplatePaneNodeData` row has a clip frame, the reconstructed `UiTreeNode.layout_cache.clip_frame` is seeded before `surface.rebuild()`. The shared hit grid then rejects pointer hits outside the same effective clip used by painting.

## Authoring Contract

Scrollable showcase regions must be authored as real shared scroll containers:

```toml
component = "ScrollableBox"
layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 10.0, scrollbar_visibility = "Auto" } }
```

A node whose component name is `ScrollableBox` but whose layout container is `VerticalBox` will arrange like a linear panel and will not contribute a scroll viewport clip to descendants. The Component Showcase smoke regression uses the lower `ContextActionMenuDemo` row in a short viewport to prove that off-screen rows receive the `ComponentShowcaseScroll` clip instead of painting over the bottom log.
