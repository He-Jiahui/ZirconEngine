# Runtime74 Typed Component Event Routing

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-typed-component-event-routing.md","docs/zircon_runtime/ui/surface/default_interactions.md","docs/zircon_runtime/ui/v2.md","zircon_editor/assets/ui/editor/components/showcase/showcase_collections_section.zui","zircon_editor/assets/ui/editor/components/showcase/showcase_input_section.zui","zircon_editor/assets/ui/editor/components/showcase/showcase_selection_section.zui","zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui","zircon_editor/src/tests/editing/ui_asset_replay.rs","zircon_editor/src/tests/ui/boundary/template_assets.rs","zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs","zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs","zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs","zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs","zircon_editor/src/ui/workbench/reference/builder/nodes.rs","zircon_runtime_interface/src/ui/template/document.rs","zircon_runtime/src/ui/component/catalog/material_foundation/selection_inputs.rs","zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/tests/accessibility_widget_actions.rs","zircon_runtime/src/ui/tests/component_catalog/material_foundation/selection_inputs.rs","zircon_runtime/src/ui/tests/event_routing.rs","zircon_runtime/src/ui/tests/mod.rs","zircon_runtime/src/ui/tests/pointer_click_semantics.rs","zircon_runtime/src/ui/tests/runtime_input_manager.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs","zircon_runtime/src/ui/tests/runtime_input_reply_routes/table_pointer_routes.rs","zircon_runtime/src/ui/tests/runtime_window_input_pump.rs","zircon_runtime/src/ui/tests/shared_core/scroll_mutation/property_mutation.rs","zircon_runtime/src/ui/tests/v2_asset.rs","zircon_runtime/src/ui/tests/v2_asset/default_controls.rs","zircon_runtime/src/ui/tests/v2_asset/range_controls.rs","zircon_runtime/src/ui/tests/widget_menu_behavior.rs","zircon_runtime/src/ui/tests/widget_radio_behavior.rs","zircon_runtime/src/ui/tests/widget_range_navigation.rs","zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs","zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard_clipboard.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs","zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs","zircon_runtime/src/ui/tests/widget_text_input_mui.rs","zircon_runtime/src/ui/tests/widget_text_input_pointer.rs","zircon_runtime/src/ui/v2/compiler.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P0-003`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`binding_targets_component_event` inferred component event identity by scanning binding id, route,
action route, and action id for case-sensitive CamelCase substrings. Test fixtures used those tokens,
while product `.zui` assets use lower_snake routes. Renaming an opaque business string could
therefore suppress an intended event or accidentally select a different one.

## Scope Delivered

- `UiBindingRef::component_event` carries `Option<UiComponentEventKind>` as the serialized typed
  event handle. Missing fields remain readable and mean that the binding is not a typed selector.
- V2 compilation validates each typed handle after component instancing. Component-only assets
  compile each declared root through the complete prototype store; default compilation uses the
  Material-then-showcase descriptor precedence, and extension hosts can inject their registry.
  Unknown final components and event kinds not declared by `UiComponentDescriptor::supports_event`
  fail closed.
- The authoritative Material `Dropdown` descriptor declares the product routing contract
  (`OpenPopup`, `SelectOption`, and `ClosePopup`) in addition to keyboard and value-change events;
  its catalog regression test locks that lower-layer contract.
- Default interactions compare the enum handle directly. The CamelCase token table and every
  production scan of binding id, route, action route, and action id are removed.
- Four product assets declare 91 typed lower_snake routes: collections 23/24, input 26/29,
  selection 30/30, and workbench component drawer 12/33. Remaining entries are ordinary low-level
  UI events and are intentionally not assigned a component-event identity.
- Existing Rust construction sites explicitly use `None`; test fixtures that model typed component
  events convert their final CamelCase path segment in test-only code.

## Architecture Evidence

- Fyrox's UI routes typed message enums through message data rather than handler-name strings; this
  is the dominant implementation reference for retaining an enum identity through dispatch.
- Godot declares signal identity and payload shape explicitly and emits by declared name; this
  supports compile-time declaration rather than route-substring recovery.
- Zircon intentionally reuses its existing `UiComponentEventKind` and typed `UiComponentEvent`
  payload enum instead of introducing a second signal-id type. The descriptor already owns the
  supported-kind set, so the V2 compiler only needs to resolve and validate that existing handle.

## Deterministic Performance Evidence

For a 1,000-binding candidate set:

- legacy maximum string-field scans: `4,000`;
- optimized enum comparisons: `1,000`;
- optimized string scans: `0`;
- eliminated string scans: `4,000` (`100%`).

The runtime marker is
`PERF-RUNTIME74-TYPED-EVENT sample_bindings=1000 legacy_string_field_scans=4000 optimized_enum_comparisons=1000 string_scans_eliminated=4000 matched_bindings=1`.
This is deterministic operation-count evidence; no wall-clock speedup is claimed.

## TDD And Validation State

- `typed_component_event_serde_round_trips_declared_identity` covers typed and legacy serialized
  forms.
- `typed_component_event_compile_rejects_unsupported_descriptor_event` locks descriptor authority.
- `typed_component_event_routing_ignores_string_spelling` covers deceptive similar substrings and
  id/route/action renames.
- `typed_component_event_hot_path_eliminates_string_scans` covers 1,000 candidates and emits the
  performance marker.
- `typed_component_event_product_assets_declare_lower_snake_routes` loads all four migrated product
  asset families through `UiV2PrototypeStoreFileCache`, compiles their imports and all 91 typed
  declarations, and locks representative lower_snake route mappings including `Tab`/`Tabs` using
  descriptor-owned `ValueChanged` rather than route-owned `select_option` wording.
- Focused Cargo tests and grouped external validation are pending. No Cargo pass is claimed.

## Remaining Scope

This closes only `RTB-P0-003`. Instance-qualified control references, model/command transactions,
generation-qualified subscriptions, transactional hot reload, and richer versioned event schema
metadata remain open under Runtime74.
