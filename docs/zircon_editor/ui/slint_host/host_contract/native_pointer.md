---
related_code:
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/workbench/layout/drawer_attachment.rs
implementation_files:
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
plan_sources:
  - user: 2026-05-07 editor input, split bar drag, drawer drag selection, and single-select regressions
tests:
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/workbench/layout/drawer_attachment.rs
  - cargo test -p zircon_editor native_host_binding_only_template_text_field_accepts_keyboard_input -- --nocapture (blocked by pre-existing zircon_editor lib-test compile errors in runtime_report_state.rs and diagnostics/localization.rs)
doc_type: module-detail
---

# Native Pointer Host Contract

The native Slint host contract is the editor-side boundary that translates root window pointer and keyboard input into editor shell callbacks. `native_pointer.rs` owns hit routing for top-level chrome, pane surfaces, template nodes, tab dragging, and shell resize splitters. `window.rs` owns keyboard/text dispatch after `native_pointer.rs` has established a focused text-input target.

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

This is a capture-style contract: after the user presses the split bar, later move/up events do not need to remain physically over the splitter hitbox. The layout resize implementation in the Slint app can therefore receive the complete down/move/up sequence needed to adjust drawer and workspace bounds.

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

At the time this document was written, focused `zircon_editor` tests were blocked before execution by existing lib-test compile errors in unrelated asset-editor files:

- `zircon_editor/src/ui/asset_editor/session/runtime_report_state.rs` shadows the `resource_diagnostic_items(...)` helper with a local vector.
- `zircon_editor/src/ui/asset_editor/diagnostics/localization.rs` borrows a localization diagnostic while moving fields out of it.
