---
related_code:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/catalog.rs
  - zircon_runtime/src/ui/component/category.rs
  - zircon_runtime/src/ui/component/descriptor.rs
  - zircon_runtime/src/ui/component/drag.rs
  - zircon_runtime/src/ui/component/event.rs
  - zircon_runtime/src/ui/component/state.rs
  - zircon_runtime/src/ui/component/validation.rs
  - zircon_runtime/src/ui/component/value.rs
  - zircon_runtime/src/ui/binding/model/event_kind.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/component_showcase.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/reference_component_tests.rs
  - zircon_editor/src/ui/slint_host/ui/structure_component_tests.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_surface_host_context.slint
  - zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs
  - zircon_editor/src/ui/workbench/autolayout/constraints/defaults.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/template_node_data.slint
  - zircon_editor/ui/workbench/template_pane.slint
  - zircon_editor/build.rs
  - zircon_editor/assets/ui/editor/component_showcase.ui.toml
  - zircon_editor/assets/ui/editor/component_widgets.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
implementation_files:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/catalog.rs
  - zircon_runtime/src/ui/component/category.rs
  - zircon_runtime/src/ui/component/descriptor.rs
  - zircon_runtime/src/ui/component/drag.rs
  - zircon_runtime/src/ui/component/event.rs
  - zircon_runtime/src/ui/component/state.rs
  - zircon_runtime/src/ui/component/validation.rs
  - zircon_runtime/src/ui/component/value.rs
  - zircon_runtime/src/ui/binding/model/event_kind.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/component_showcase.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/reference_component_tests.rs
  - zircon_editor/src/ui/slint_host/ui/structure_component_tests.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_surface_host_context.slint
  - zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs
  - zircon_editor/src/ui/workbench/autolayout/constraints/defaults.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/template_node_data.slint
  - zircon_editor/ui/workbench/template_pane.slint
  - zircon_editor/build.rs
  - zircon_editor/assets/ui/editor/component_showcase.ui.toml
  - zircon_editor/assets/ui/editor/component_widgets.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
plan_sources:
  - user: 2026-04-27 Runtime UI 组件库与 Slint Material Showcase Cutover
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
tests:
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime/src/ui/tests/mod.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/pane_template_descriptor.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-showcase-check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_runtime --lib drop_binding_roundtrip_preserves_reference_payload_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-showcase-check -- --nocapture
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-showcase-check -- --nocapture
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib builtin_activity_window_documents_are_registered_in_host_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib builtin_pane_views_expose_template_metadata --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib showcase_demo_state_exercises_full_component_action_bindings --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library-editor -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library-editor -- --nocapture
  - cargo test -p zircon_editor --lib showcase_demo_state_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase_pane_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_numeric_drag_tracks_two_axis_delta --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_popup_options_dispatch_candidate_selection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_action_chips_dispatch_secondary_actions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_materializes_visual_feedback_and_vector_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_materializes_reference_drop_wells --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_materializes_structure_and_collection_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib host_projection_carries_runtime_component_properties_and_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib slint_host_build_uses_material_style --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib builtin_activity_windows_expose_window_template_documents --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
doc_type: module-detail
---

# Runtime UI Component Showcase

## Scope

This document records the first retained Runtime UI component-library slice. The goal is to make editor screen-space UI components explicit runtime semantics while keeping `.ui.toml` as the business source of truth and keeping Slint as a generic Material host.

The V1 acceptance target is not a visual mock. The showcase asset declares real component nodes, the runtime owns descriptors and typed event/state contracts, and host projection carries enough metadata for a generic renderer to show component role, value, validation, popup, selection, and drag/drop acceptance state. Interactive showcase rows also declare registered template bindings so Button, NumberField, Dropdown, Foldout, Array, Map, and reference-drop controls have concrete event entry points.

## Runtime Component Contracts

`zircon_runtime::ui::component` is the shared contract layer for editor-style controls. It defines:

- `UiComponentDescriptorRegistry`: the registry used by the editor showcase and host projection.
- `UiComponentDescriptor`: component id, category, Material-style role, typed prop schema, state schema, slot schema, supported events, and drop policy.
- `UiPropSchema`, `UiOptionDescriptor`, and `UiSlotSchema`: typed declaration metadata for props, choices, and content slots.
- `UiValue` / `UiValueKind`: bool, int, float, string, color, vec2/3/4, asset reference, instance reference, array, map, enum, flags, and null values.
- `UiComponentEvent`: retained/event-driven control events such as value changed, commit, focus, drag delta and large-step drag delta, popup open/close, option selection, foldout toggle, array element add/set/remove/move, map add/set/remove, typed reference drop, and reference clear/locate/open actions.
- `UiComponentState`: demo-retained state that applies events, honors per-control numeric min/max/step/large-step state, records validation errors, tracks focus/drag/popup/expanded flags, rejects disabled selection options, supports dropdown-style multi-selection arrays, rejects duplicate and missing map keys, and enforces drag/drop policy with rejection messages.

The registry currently includes the V1 showcase set:

- Visual and feedback: `Label`, `RichLabel`, `Image`, `Icon`, `Separator`, `ProgressBar`, `Spinner`, `Badge`, `HelpRow`.
- Input: `Button`, `IconButton`, `ToggleButton`, `Checkbox`, `Radio`, `SegmentedControl`, `InputField`, `TextField`.
- Numeric: `NumberField`, `RangeField`, `ColorField`, `Vector2Field`, `Vector3Field`, `Vector4Field`.
- Selection: `Dropdown`, `ComboBox`, `EnumField`, `FlagsField`, `SearchSelect`.
- Reference: `AssetField`, `InstanceField`, `ObjectField`.
- Structure and collection: `Group`, `Foldout`, `PropertyRow`, `InspectorSection`, `ArrayField`, `MapField`, `ListRow`, `TreeRow`, `ContextActionMenu`.

`SegmentedControl` is intentionally modeled as direct selection rather than popup selection. It supports focus and option selection state, but does not expose `popup_open`, `OpenPopup`, or `ClosePopup`; dropdown-like metadata remains on `Dropdown`, `ComboBox`, `EnumField`, `FlagsField`, `SearchSelect`, and `ContextActionMenu`.

## Showcase Assets

The showcase window is declared in `zircon_editor/assets/ui/editor/component_showcase.ui.toml`. It imports `component_widgets.ui.toml` for the reusable `ShowcaseSection` widget and `editor_material.ui.toml` for style tokens.

The window layout follows a Rider/Unity inspector shape:

- left category navigation with stable control ids for visual/input/collection groups;
- center scroll area with Visual, Input/Numeric, Selection/Reference, and Collection/Inspector groups;
- right state panel with retained event examples and payload hints.

Every V1 component has a native node row in the showcase. Those rows deliberately remain `.ui.toml` data. There is no component-specific `.slint` tree for the showcase.

Interactive rows use `[[...bindings]]` in the `.ui.toml` asset and matching entries in `template_bindings.rs`. The binding payloads are `EditorUiBindingPayload::Custom(UiBindingCall::new("UiComponentShowcase"))` with the demo action and control id as arguments. NumberField drag and large-step drag, RangeField change, popup open/close, dropdown option selection, reference drop/clear/locate/open, Array add/set/remove/move, and Map add/set/remove actions are all declared by the asset and registry rather than by a handwritten showcase `.slint` tree. This keeps the showcase event surface event-driven and retained without teaching Slint any showcase-specific business layout.

`EditorUiHostRuntime` owns the showcase transient state through `showcase_demo_state.rs`. The reducer consumes the projected binding payloads, maps them to typed `UiComponentEvent` values, applies them to retained `UiComponentState`, and records an event log entry with the control id and changed display value. The reducer covers category selection, button commit, text/value change, toggles, numeric drag begin/update/end and large-step drag, popup open/close, dropdown selection, typed reference drops, reference clear/locate/open actions, foldout/group expansion, Array add/set/remove/move, and Map add/set/remove operations.

The retained state is projected back onto the host model before Slint conversion. `runtime_host.rs` asks the showcase state to overlay current `value`, `items`, `entries`, generated `collection_items`, `expanded`, `checked`, `focused`, `dragging`, `popup_open`, validation, and event-log text onto matching host nodes. A fresh projection after an event therefore shows the updated NumberField value, focused list row state, selected dropdown value, open or closed popup state, dropped reference, generated Array/Map child rows, collection counts, and event log without mutating the `.ui.toml` structure.

`UiEventKind::Drop` maps to `onDrop` so `AssetField`, `InstanceField`, and `ObjectField` can expose reference-drop semantics directly rather than tunneling through a click or change event.

## Slint Host Boundary

The Slint build style is switched to Material in `zircon_editor/build.rs`. The `.slint` files continue to act as host primitives and generated DTO surfaces.

Two host paths now carry runtime component semantics:

- `EditorUiHostRuntime -> SlintUiHostAdapter` exposes component role, value text, validation, popup state, selection state, option summary text, individual option ids, checked/expanded/disabled, and accepted drag payload metadata for tests and host-level projections.
- `pane_data_conversion.rs` derives the same component metadata for generic `TemplatePaneNodeData`, including both popup option summary text and the individual candidate list. The generic `template_pane.slint` can therefore draw runtime component rows, popup candidate rows, and numeric drag action metadata without knowing showcase-specific structure.

`template_pane.slint` only adds generic component-row rendering. It does not contain the list of showcase controls, groups, labels, or demo values.

`UiComponentShowcase` now also has a pane-body payload and template metadata, so a docked Activity Window is rendered by `TemplatePane` instead of falling through to `FallbackPane`. The pane conversion path uses `editor.window.ui_component_showcase`, computes layout for the available pane size, and forwards the resulting runtime component nodes to the generic Slint template pane.

The Slint host owns a dedicated `EditorUiHostRuntime` for the showcase transient state. During host startup, `host_lifecycle.rs` loads the builtin Runtime UI templates into that runtime, and `apply_presentation.rs` threads the runtime into docked panes and native floating panes before converting Runtime UI nodes into `TemplatePaneNodeData`. The default scene projection path remains generic; the supplied runtime is only used to overlay retained demo values onto the Runtime UI projection.

Generic component rows now carry `dispatch_kind = "showcase"` and the selected binding action id when the projected runtime node exposes a `UiComponentShowcase/...` binding. Popup-capable controls prefer an `OpenPopup` action while closed and switch to their `SelectOption` action once the retained state says the popup is open. Their TOML `options` array is projected as a Slint string model, so the popup layer renders individual candidate rows rather than only a single summary label. `ContextActionMenu` can also project a retained `menu_items` model with checked, disabled, separator, and shortcut metadata encoded in `.ui.toml`; the generic popup layer renders those menu rows without embedding showcase menu content in `.slint`. Candidate row clicks dispatch the selected option id through `PaneSurfaceHostContext.component_showcase_option_selected`, and `callback_wiring.rs` forwards that generic option event into Rust host state. Numeric rows carry `begin_drag_action_id`, `drag_action_id`, and `end_drag_action_id`, so pointer down/update/up can dispatch `BeginDrag`, `DragDelta`, and `EndDrag` through the same retained event reducer while ordinary activation remains event-driven. Editable fields carry a separate `edit_action_id`, allowing InputField, TextField, NumberField, and RangeField rows to use a generic `LineEdit` and dispatch live value text through the same `.ui.toml` binding and retained reducer path. Multi-operation rows also carry generic `TemplatePaneActionData` chips, so AssetField exposes Find/Open/Clear, ArrayField exposes Add/Set/Remove/Move, and MapField exposes Add/Set/Remove directly in the showcase window. `TemplatePaneNodeData` also projects `value_number`, normalized `value_percent`, parsed `value_color`, `focused`, `hovered`, `pressed`, `dragging`, `drop_hovered`, generated `collection_items`, and menu metadata, allowing the retained host to render Material-like checkbox, radio, switch, progress bar, range track, color swatch, list focus, tree/collection summaries, and reference drop-hover wells without hardcoding the showcase component list into `.slint`. It now also projects `media_source`, `icon_name`, and typed `vector_components`, so Image, Icon, SvgIcon, Separator, Spinner, Badge, HelpRow, and Vector2/3/4 rows render as retained host primitives instead of falling back to plain value text. Reference controls use the existing retained metadata as a Material drop well: AssetField, InstanceField, and ObjectField show the current reference value, accepted drag payload kinds, validation or rejected-drop messages, and action chips without requiring new business `.slint` or duplicate projection fields. Structure and collection controls now use the same retained surface: Group, Foldout, InspectorSection, PropertyRow, ArrayField, MapField, ListRow, and TreeRow render disclosure/summary chrome from `expanded`, `value_text`, `selection_state`, generated child-row metadata, and action-chip metadata instead of plain value text. `template_pane.slint` has a generic activation, edit, drag, option-selection, action-chip, and role-specific primitive surface for component rows; it tracks two-dimensional pointer movement and maps right/up movement to positive deltas and left/down movement to negative deltas for NumberField and RangeField style controls. `pane_content.slint` routes showcase activations through `PaneSurfaceHostContext.component_showcase_control_activated`, edits through `component_showcase_control_edited`, drag deltas through `component_showcase_control_drag_delta`, popup option selections through `component_showcase_option_selected`, and secondary action chips back through `component_showcase_control_activated`; `callback_wiring.rs` forwards those callbacks into Rust host state.

`pane_surface_actions.rs` maps the generic control activation, live edit, drag delta, option selection, or secondary action chip back to the projected `EditorUiBinding`, builds the typed demo input, applies `showcase_demo_state.rs` through `EditorUiHostRuntime::apply_showcase_demo_binding`, marks the presentation dirty, and lets the next retained projection redraw the updated value or validation message. Numeric live edits parse valid number text into float values and pass invalid text through as a string so the retained numeric validation path can surface the error. Option selections pass the clicked candidate id into `UiComponentShowcaseDemoEventInput::SelectOption`, so Dropdown/ComboBox/Enum/Flags/SearchSelect/ContextActionMenu rows use the same runtime reducer as non-hosted tests. Secondary action chips reuse the existing binding ids for reference clear/locate/open, Array add/set/remove/move, and Map add/set/remove operations. Slint therefore remains a generic Material host, while `.ui.toml` and the runtime reducer remain the business truth for the showcase interaction.

## Validation Coverage

Runtime tests cover registry completeness, direct segmented-control selection semantics, popup selection semantics, NumberField drag/clamp, large-step drag and invalid commit validation, disabled selection option rejection with retained validation state, special selection option metadata, Dropdown multi-selection, FlagsField retained selection state, Array add/set/remove/move, Map add/set/remove with duplicate and missing-key rejection, and AssetField/InstanceField/ObjectField typed drop plus reference clear/locate/open semantics.

Editor tests cover:

- `editor.ui_component_showcase` registration as an Activity Window with the correct template document id;
- builtin template loading and projection of `editor.window.ui_component_showcase`;
- runtime component metadata in Slint host projection for NumberField, Dropdown, and AssetField, including editable `edit_action_id`, numeric `drag_action_id`, popup option summary text, structured option ids, and individual popup candidate rows for dropdown-like controls;
- materialized primitive metadata and rendering hooks for Checkbox, Radio, ToggleButton, ProgressBar, RangeField, and ColorField, including projected `value_number`, normalized `value_percent`, and parsed `value_color`;
- materialized primitive metadata and rendering hooks for Image, Icon, SvgIcon, Separator, Spinner, Badge, HelpRow, and Vector2/3/4 fields, including projected `media_source`, `icon_name`, and typed vector component models;
- Material reference drop-well rendering for AssetField, InstanceField, and ObjectField, including accepted drag payload metadata, current reference display, validation/rejection message display, and action-chip placement that does not cover the reference value;
- Inspector-style structure and collection row rendering for Group, Foldout, InspectorSection, PropertyRow, ArrayField, MapField, ListRow, and TreeRow, including retained expanded/collapsed state, value summaries, and action-chip placement beside collection summaries;
- deeper retained structure coverage for generated Array/Map child rows, ListRow focused selection-state projection, NumberField dragging projection, ContextActionMenu menu-row metadata, and TemplatePane rendering hooks for `focused`, `dragging`, `drop_hovered`, `collection_items`, and `menu_items`;
- showcase binding projection for NumberField drag/commit, Dropdown change, popup open/close, and AssetField drop, including the custom runtime demo action payload;
- retained showcase demo state application for projected bindings, including category selection, input value mutation, toggle state, numeric drag and large-step drag, popup open/close, dropdown selection, typed asset drop, reference clear/locate/open, Array add/set/move/remove, Map add/set/remove, and event-log recording;
- docked `UiComponentShowcase` pane conversion to Slint `TemplatePane` nodes so the showcase uses the Runtime UI template instead of a fallback pane;
- Material style selection in the Slint build script.

## Recent Validation

On 2026-04-27, the focused showcase cutover checks passed with `--locked` and `--jobs 1` against `D:\cargo-targets\zircon-codex-editor-check-tests`:

- `cargo test -p zircon_editor --lib component_showcase`
- `cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics`
- `cargo test -p zircon_editor --lib component_showcase_template_popup_options_dispatch_candidate_selection`
- `cargo test -p zircon_editor --lib component_showcase_template_action_chips_dispatch_secondary_actions`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_runtime_component_primitives`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_visual_feedback_and_vector_primitives`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_reference_drop_wells`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_structure_and_collection_rows`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane`
- `cargo test -p zircon_editor --lib component_showcase_template_fields_dispatch_live_edits`
- `cargo test -p zircon_runtime --lib ui::tests::component_catalog`
- `cargo test -p zircon_editor --lib slint_host_build_uses_material_style`
- `cargo check -p zircon_editor --lib`
- `cargo test -p zircon_editor --lib builtin_activity_windows_expose_window_template_documents`
- `cargo test -p zircon_editor --lib host_projection_carries_runtime_component_properties_and_routes`
- `cargo test -p zircon_editor --lib builtin_activity_window_documents_are_registered_in_host_runtime`
- `cargo test -p zircon_editor --lib builtin_pane_views_expose_template_metadata`

During this validation loop, an intermediate compile attempt briefly observed a mismatched `store_last_runtime_outputs` signature while parallel runtime graphics work was changing the same support layer. A fresh rerun after that support-layer source settled passed the showcase and editor checks above.

On 2026-04-28, the deeper retained-state slice added tests for collection child-row projection and generic menu/control-state rendering, but the local Windows cargo process repeatedly exited with code `-1` after long `zircon_runtime` / `zircon_editor` compile phases without emitting a Rust or Slint diagnostic. Multiple residual `cargo` / `rustc` processes were cleared and a heartbeat wrapper still ended with stderr stopping at `Compiling zircon_editor`. The new checks therefore remain pending until cargo process stability is restored:

- `cargo test -p zircon_editor --lib showcase_demo_state_projects_collection_children_and_control_flags --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_deep_collection_and_menu_state --locked --jobs 1`

At this stage, asset and instance drag/drop are semantic contracts and demo transient state. Real asset browser and scene tree source integration is coordinated by the active `runtime-ui-drag-source-metadata` session and can attach later through the existing editor data sources without changing the component descriptor layer.
