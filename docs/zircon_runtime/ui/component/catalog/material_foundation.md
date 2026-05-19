---
related_code:
  - zircon_editor/assets/ui/editor/material_components/material_autocomplete.zui
  - zircon_editor/assets/ui/editor/material_components/material_checkboxes.zui
  - zircon_editor/assets/ui/editor/material_components/material_number_field.zui
  - zircon_editor/assets/ui/editor/material_components/material_radio_buttons.zui
  - zircon_editor/assets/ui/editor/material_components/material_selects.zui
  - zircon_editor/assets/ui/editor/material_components/material_switches.zui
  - zircon_editor/assets/ui/editor/material_components/material_text_fields.zui
  - zircon_editor/assets/ui/editor/material_components/material_textarea_autosize.zui
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/panel_preset.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/selection_inputs.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/text_inputs.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/selection_inputs.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_editor/assets/ui/editor/fyrox_panel_demo_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/layout_demo_window.v2.ui.toml
  - zircon_runtime_interface/src/ui/skin/preset.rs
implementation_files:
  - zircon_editor/assets/ui/editor/material_components/material_autocomplete.zui
  - zircon_editor/assets/ui/editor/material_components/material_checkboxes.zui
  - zircon_editor/assets/ui/editor/material_components/material_number_field.zui
  - zircon_editor/assets/ui/editor/material_components/material_radio_buttons.zui
  - zircon_editor/assets/ui/editor/material_components/material_selects.zui
  - zircon_editor/assets/ui/editor/material_components/material_switches.zui
  - zircon_editor/assets/ui/editor/material_components/material_text_fields.zui
  - zircon_editor/assets/ui/editor/material_components/material_textarea_autosize.zui
  - zircon_runtime/src/ui/component/catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/selection_inputs.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/text_inputs.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_editor/assets/ui/editor/fyrox_panel_demo_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/layout_demo_window.v2.ui.toml
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
  - .codex/plans/ZirconEngine UI 组件化与 Material 样式器重构计划.md
  - docs/superpowers/plans/2026-05-17-mui-all-components-detailed-design.md
tests:
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/selection_inputs.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/mod.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/checkbox.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/radio.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inputs/switch.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/feedback.rs
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib material_editor_foundation_catalog_covers_planned_component_layers --locked --jobs 1 --message-format short --color never (2026-05-18: passed, 1 passed)
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib material --locked --offline --jobs 1 --message-format short --color never (2026-05-18: passed, 68 passed)
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - cargo check -p zircon_runtime --lib --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed)
  - cargo test -p zircon_runtime --lib material_editor_foundation_catalog_covers_planned_component_layers --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime --locked --verbose (2026-05-12 runtime closeout: passed after `TreeView.expanded` schema coverage was restored; runtime lib target reported 1268 passed, 0 failed)
  - cargo test -p zircon_runtime --lib layout_demo_window_compiles_with_window_drawer_and_data_view_components --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime --lib fyrox_panel_demo_window_compiles_with_all_panel_role_components --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib default_stack --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 8 passed)
doc_type: module-detail
---

# Material Editor Foundation Catalog

`UiComponentDescriptorRegistry::material_editor_foundation()` is the first component catalog for the new editor UI direction. It is intentionally separate from `editor_showcase()` so existing showcase tests and legacy showcase consumers keep their exact component set while the new workbench path gains a clean Material-first contract.

## Component Layers

The catalog is organized around the implementation plan's four layers:

- primitive controls: button, icon button, text field, checkbox, switch, dropdown, slider, tabs, menu, tooltip, scrollbar, splitter, panel, modal;
- layout controls: flex group, horizontal group, vertical group, grid group, overlay, scroll view;
- data/view controls: list view, virtual list, tree view, property grid, inspector section;
- Fyrox panel role controls: search field, context menu, field editor, folder tree, asset grid/list, preview and metadata panes, viewport host, pane toolbar, gizmo controls, filter bar, severity chips, categorized list, status action controls, graph canvas, source editor, timeline, and visual designer;
- shell controls: window, view, drawer, view tab, window frame, document node, tab stack, floating window, dock host, workbench shell, slot, and composite.

Every descriptor receives the `material_dark`, `material`, and `material-dark` default classes. Each descriptor also declares the same visual state schema used by the Material skin: hovered, pressed, selected, disabled, focused, error, and warning.

## Behavior Contracts

The foundation descriptors now carry the first behavior-level props and events needed by the planned Workbench shell:

- `VirtualList` declares `item_count`, `item_extent`, `overscan`, and `SetVisibleRange`.
- `TreeView` declares `query` and `expanded` props plus selection, expand/collapse, and context-menu events.
- `PropertyGrid` and `InspectorSection` keep inspector-style section/field slots and property-edit events.
- `SearchField`, `FilterBar`, and `SeverityChips` declare query/severity props and selection/value-change events for Hierarchy, Asset Browser, Console, and Diagnostics.
- `FolderTree`, `AssetGrid`, `AssetList`, and `CategorizedList` declare collection slots plus selection, expand/collapse, context-menu, locate, and open-reference events.
- `PreviewPane`, `MetadataPane`, `FieldEditor`, and `StatusActionControls` give Inspector, Asset Browser, Plugin Manager, and export views reusable editor-only composite surfaces.
- `ViewportHost`, `GraphCanvas`, and `VisualDesigner` are canvas-layout components for Scene/Game, Material, UI, and animation authoring surfaces.
- `PaneToolbar`, `GizmoControls`, `SourceEditor`, and `Timeline` cover viewport/editor toolbars, transform controls, text-source editing, and animation scrubbing.
- `DocumentNode`, `TabStack`, and `FloatingWindow` expose the Workbench split/tab/detach model as authorable shell components.
- `Drawer`, `View`, `ViewTab`, `Window`, `WindowFrame`, `DockHost`, and `WorkbenchShell` declare stable ids, active-selection/focus props, preset ids, and shell interaction events.
- `ButtonGroup` is a structural composite: it declares group orientation, attached-radius, segment-count, and disabled-propagation metadata plus the child button style schemas/defaults (`button_variant`, `button_color`, `button_size`, and `icon_placement`). The group itself stays non-dispatchable; child buttons own click, press, focus, and selection routes.

These are still neutral component descriptors. They do not create native windows or bind editor state directly; they let v2 assets and editor presenters agree on which values and events are valid before a concrete skin or host renders them.

Text-backed Material descriptors live in the `text_inputs` child module instead of the broader input-control module. That subtree owns `TextField`, `Input`, `TextareaAutosize`, `SearchField`, `FieldEditor`, and `SourceEditor` because they share the `TextInput` host capability and edit/commit event family. `TextField` currently records the MUI wrapper split as metadata: `variant = outlined|filled|standard`, `label`, `value_text`, `placeholder`, `helper_text`, `multiline`, and `select_mode`. The Material Lab prototype freezes one representative route-bearing row plus non-dispatchable visual children for outlined focus, filled helper text, standard underline, error helper, and disabled no-edit, while live helper-label layout and multiline/select-mode behavior remain later runtime support work. `TextareaAutosize` uses the same owner module and adds layout-affecting row metadata: `variant`, `value_text`, `placeholder`, `helper_text`, `multiline`, `autosize`, `min_rows`, and `max_rows`. Its prototype keeps one change route on the sample row and uses visual-only child examples for minimum rows, maximum row clamp, focused autosize, error helper, and disabled no-edit so later layout tests can distinguish row-clamp metadata from render-only field styling.

`NumberField` remains in the numeric/input owner module because it shares drag and clamp behavior with runtime numeric state rather than the `TextInput` host capability. The Material foundation descriptor uses the field-sized numeric schema (`value` range `0..100`, step `1`) and exposes typed numeric `value`, `min`, `max`, `step`, and `large_step` defaults, plus `Focus`, `BeginDrag`, `DragDelta`, `LargeDragDelta`, `EndDrag`, `ValueChanged`, and `Commit`. Its Material Lab prototype keeps one route-bearing sample row for `MaterialLab/NumberField/Change`; the visible child examples are non-dispatchable `NumberField` nodes for stepper, clamped max, drag-active chip, error, and disabled states. The slice intentionally does not add string-only `value_text` to the numeric children, so retained projection continues to read `value_number` from the numeric `value` prop.

`Checkbox` remains in the basic input owner module because its runtime behavior is a local toggle value rather than a popup selection list. The descriptor exposes `text`, `checked`, `indeterminate`, `label_click_toggles`, and `indeterminate_resolves_to_checked`, plus `Focus` and `ValueChanged`. The current policy records that clicking an indeterminate checkbox resolves to checked. Its Material Lab prototype keeps one route-bearing sample row for `MaterialLab/Checkboxes/Toggle`; the visible child examples are non-dispatchable `Checkbox` nodes for unchecked, checked, indeterminate, error, and disabled states. Child examples keep `label_click_toggles = true` so later headless behavior tests can assert that label and box activation share the same disabled-aware owner.

`Radio` remains in the basic input owner module, but the descriptor now carries the group metadata needed by MUI `RadioGroup`: `group_value`, `option_id`, `options`, `disabled_options`, `label_click_selects`, `exclusive_group`, and `keyboard_navigation`. It exposes `Focus`, `SelectOption`, and `ValueChanged` so later behavior tests can separate focus movement from committing a single selected option. The Material Lab prototype keeps one route-bearing sample row for `MaterialLab/RadioButtons/Change`; the visible child examples are non-dispatchable `Radio` nodes for selected, unselected, disabled, and error options. Each child freezes the same option list and selected group value so the later runtime implementation can prove exclusive selection clears previous options and disabled options are rejected without changing the prototype schema.

`Switch` remains in the basic input owner module because it is a local checked toggle with track/thumb presentation rather than grouped selection or popup state. The descriptor exposes `text`, `checked`, `switch_size`, `switch_color`, `label_click_toggles`, `track_click_toggles`, and `thumb_draggable`, plus `Focus` and `ValueChanged`. The current policy records label and track clicks as toggle owners while thumb dragging stays disabled until drag semantics are implemented. Its Material Lab prototype keeps one route-bearing sample row for `MaterialLab/Switches/Toggle`; the visible child examples are non-dispatchable `Switch` nodes for on, off, small, disabled, and error states. Child examples freeze the size/color and toggle policy metadata so later runtime behavior can prove disabled no-toggle and local checked-state mutation without changing the prototype schema.

Selection-backed Material descriptors live in the `selection_inputs` child module instead of the broader input-control module. That subtree owns `Select`, `Dropdown`, `Autocomplete`, and `ToggleButtonGroup` because they share popup/option or grouped-selection ownership rather than text-edit or numeric-drag ownership. `Select` now records the Material field variant, display value, selected option ids, option ids, disabled/focused/hovered/pressed option id sets, `multiple`, `display_empty`, and `popup_open` metadata. It exposes `Focus`, `OpenPopup`, `SelectOption`, `ClosePopup`, and `ValueChanged` so later runtime popup support can distinguish opening the menu from committing a selected option. Its Material Lab prototype keeps one route-bearing sample row for `MaterialLab/Selects/Change`; the visible child examples are non-dispatchable `Select` nodes for closed placeholder, open popup, selected option, multi-chip selection, and disabled option states.

`Autocomplete` stays in `selection_inputs` but adds the search-query side of the popup contract. Its descriptor records `query`, display value, selected option ids, all option ids, filtered option ids, disabled/focused/hovered/pressed/matched option id sets, `multiple`, `free_solo`, and `popup_open` metadata. It exposes `Focus`, `ValueChanged`, `OpenPopup`, `SelectOption`, `ClosePopup`, and `RemoveElement`; the remove event is reserved for chip deletion while query edits and option commits remain distinct. Its Material Lab prototype keeps one route-bearing sample row for `MaterialLab/Autocomplete/Change`; the visible child examples are non-dispatchable `Autocomplete` nodes for query-filtered results, open popup, selected option, multi-chip selection, and disabled option states. Free-solo and async loading are still future runtime behavior, but the descriptor carries the `free_solo` flag now so assets can declare the supported mode without changing the schema later.

`FyroxPanelComponentRole::component_id()` maps editor panel roles onto these descriptor ids. The editor-side default stack test walks every panel preset role and verifies that the Material foundation catalog contains the matching descriptor. Adding a new Fyrox panel role now requires either reusing an existing component id or adding a catalog descriptor with matching host/render capabilities.

## Host Capabilities

Editor shell descriptors require `UiHostCapability::Editor`, which keeps them out of runtime-basic host palettes. `VirtualList` requires both `UiHostCapability::VirtualizedLayout` and `UiRenderCapability::VirtualizedLayout`, preserving the existing validation invariant that virtualized rendering must be backed by a virtualized host.

Editor panel controls also require `UiHostCapability::Editor`. Text-backed controls such as `SearchField` and `SourceEditor` require `TextInput`; canvas-backed controls such as `ViewportHost`, `GraphCanvas`, and `VisualDesigner` require `CanvasRender`; preview panes require `ImageRender`.

## Layout Roles

The catalog uses `UiComponentLayoutRole` rather than renderer-specific type checks:

- flex, horizontal, vertical, and scroll views remain flex-family layouts for the Taffy bridge;
- grid group maps to grid;
- overlay stays Zircon-owned;
- virtual list stays explicitly virtualized;
- viewport, graph, and designer surfaces use the canvas layout role;
- dock host uses the editor dock role for future JetBrains/Unreal workbench shell assembly.
