---
related_code:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/descriptor/mod.rs
  - zircon_runtime/src/ui/component/descriptor/validation.rs
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/component/state_reducer/button.rs
  - zircon_runtime/src/ui/component/state_reducer/collection.rs
  - zircon_runtime/src/ui/component/state_reducer/disclosure.rs
  - zircon_runtime/src/ui/component/state_reducer/interaction.rs
  - zircon_runtime/src/ui/component/state_reducer/numeric.rs
  - zircon_runtime/src/ui/component/state_reducer/overlay.rs
  - zircon_runtime/src/ui/component/state_reducer/reference.rs
  - zircon_runtime/src/ui/component/state_reducer/selection.rs
  - zircon_runtime/src/ui/component/state_reducer/world.rs
  - zircon_runtime/src/ui/component/state_reducer/windowing.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation
  - zircon_runtime/src/ui/icon_atlas
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives
  - zircon_editor/assets/ui/editor/material_components
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/component/state_reducer/button.rs
  - zircon_runtime/src/ui/component/state_reducer/collection.rs
  - zircon_runtime/src/ui/component/state_reducer/disclosure.rs
  - zircon_runtime/src/ui/component/state_reducer/interaction.rs
  - zircon_runtime/src/ui/component/state_reducer/numeric.rs
  - zircon_runtime/src/ui/component/state_reducer/overlay.rs
  - zircon_runtime/src/ui/component/state_reducer/reference.rs
  - zircon_runtime/src/ui/component/state_reducer/selection.rs
  - zircon_runtime/src/ui/component/state_reducer/world.rs
  - zircon_runtime/src/ui/component/state_reducer/windowing.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation
  - zircon_editor/assets/ui/editor/components/workbench/primitives
  - zircon_editor/assets/ui/editor/material_components
plan_sources:
  - user: 2026-06-12 implement the editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/complex_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/button.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/windowing.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\selection.rs zircon_runtime\src\ui\component\state_reducer\collection.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-collection-family-0612-coremin --message-format short --color never (passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\selection.rs zircon_runtime\src\ui\component\state_reducer\collection.rs zircon_runtime\src\ui\component\state_reducer\reference.rs (passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-reference-family-0612-coremin --message-format short --color never (passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\selection.rs zircon_runtime\src\ui\component\state_reducer\collection.rs zircon_runtime\src\ui\component\state_reducer\reference.rs zircon_runtime\src\ui\component\state_reducer\numeric.rs (passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-reference-family-0612-coremin --message-format short --color never (passed with existing warnings after numeric-family split)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\selection.rs zircon_runtime\src\ui\component\state_reducer\collection.rs zircon_runtime\src\ui\component\state_reducer\reference.rs zircon_runtime\src\ui\component\state_reducer\numeric.rs zircon_runtime\src\ui\component\state_reducer\overlay.rs zircon_runtime\src\ui\tests\component_catalog\component_state.rs zircon_runtime\src\ui\tests\component_catalog\component_state\overlay.rs (passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-reference-family-0612-coremin --message-format short --color never (passed with existing warnings after overlay-family split)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\selection.rs zircon_runtime\src\ui\component\state_reducer\collection.rs zircon_runtime\src\ui\component\state_reducer\reference.rs zircon_runtime\src\ui\component\state_reducer\numeric.rs zircon_runtime\src\ui\component\state_reducer\overlay.rs zircon_runtime\src\ui\component\state_reducer\windowing.rs zircon_runtime\src\ui\tests\component_catalog\component_state.rs zircon_runtime\src\ui\tests\component_catalog\component_state\overlay.rs zircon_runtime\src\ui\tests\component_catalog\component_state\windowing.rs (passed)
  - git diff --check -- docs/zircon_runtime/ui/components.md docs/zircon_runtime/ui/component/state_reducer.md docs/zircon_runtime/ui/component/catalog/material_foundation.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md zircon_runtime/src/ui/component/state_reducer.rs zircon_runtime/src/ui/component/state_reducer/windowing.rs zircon_runtime/src/ui/tests/component_catalog/component_state.rs zircon_runtime/src/ui/tests/component_catalog/component_state/windowing.rs (passed with LF-to-CRLF warnings only)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-reference-family-0612-coremin --message-format short --color never (passed with existing warnings after windowing-family split)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\collection.rs zircon_runtime\src\ui\component\state_reducer\disclosure.rs zircon_runtime\src\ui\component\state_reducer\interaction.rs zircon_runtime\src\ui\component\state_reducer\numeric.rs zircon_runtime\src\ui\component\state_reducer\overlay.rs zircon_runtime\src\ui\component\state_reducer\reference.rs zircon_runtime\src\ui\component\state_reducer\selection.rs zircon_runtime\src\ui\component\state_reducer\windowing.rs zircon_runtime\src\ui\tests\component_catalog\component_state.rs zircon_runtime\src\ui\tests\component_catalog\component_state\button.rs zircon_runtime\src\ui\tests\component_catalog\component_state\overlay.rs zircon_runtime\src\ui\tests\component_catalog\component_state\windowing.rs (passed after interaction/disclosure split)
  - git diff --check -- docs/zircon_runtime/ui/components.md docs/zircon_runtime/ui/component/state_reducer.md docs/zircon_runtime/ui/component/catalog/material_foundation.md .codex/sessions/20260612-0904-editor-ui-architecture-implementation.md zircon_runtime/src/ui/component/state_reducer.rs zircon_runtime/src/ui/component/state_reducer/button.rs zircon_runtime/src/ui/component/state_reducer/collection.rs zircon_runtime/src/ui/component/state_reducer/disclosure.rs zircon_runtime/src/ui/component/state_reducer/interaction.rs zircon_runtime/src/ui/component/state_reducer/numeric.rs zircon_runtime/src/ui/component/state_reducer/overlay.rs zircon_runtime/src/ui/component/state_reducer/reference.rs zircon_runtime/src/ui/component/state_reducer/selection.rs zircon_runtime/src/ui/component/state_reducer/windowing.rs zircon_runtime/src/ui/tests/component_catalog/component_state.rs zircon_runtime/src/ui/tests/component_catalog/component_state/button.rs zircon_runtime/src/ui/tests/component_catalog/component_state/overlay.rs zircon_runtime/src/ui/tests/component_catalog/component_state/windowing.rs (passed with LF-to-CRLF warnings only after interaction/disclosure split)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-reference-family-0612-coremin --message-format short --color never (passed with existing warnings after interaction/disclosure split)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\world.rs zircon_runtime\src\ui\tests\component_catalog\complex_components.rs (passed after world-family formatting)
  - git diff --check -- zircon_runtime/src/ui/component/state_reducer.rs zircon_runtime/src/ui/component/state_reducer/world.rs (passed with LF-to-CRLF warnings only after world-family split)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-reference-family-0612-coremin --message-format short --color never (passed with existing warnings after world-family split)
  - cargo test -p zircon_runtime --lib material_button_family --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-button-family-0612-coremin --message-format short --color never -- --test-threads=1 --nocapture (timed out after 904s during lib-test compile/link with no Rust diagnostics)
doc_type: milestone-detail
---

# Runtime UI Component Inventory

This is the plan 06 M1.S1 baseline for the editor component library. It records the current L1 atom coverage against the seven Definition of Done gates from `06-component-library-mui.md`; it is an implementation inventory, not an acceptance claim.

## DoD Gates

Legend: `OK` means the current code has a usable owned implementation for the gate. `Partial` means the concept exists but still depends on another milestone, a Material showcase-only asset, or missing behavior. `Missing` means the gate has no owned implementation yet.

| Gate | Requirement | Current owner |
|---|---|---|
| Asset | A `.zui` asset exists in the Workbench primitive whitelist or the Material component showcase. | `zircon_editor/assets/ui/editor/components/workbench/primitives`, `zircon_editor/assets/ui/editor/material_components` |
| Descriptor | Props, slots, events, and state schema are declared in the runtime component catalog. | `zircon_runtime/src/ui/component/catalog/material_foundation` |
| Layout | The asset and descriptor use the plan 02 `UiLayoutStyle` path rather than private layout fields. | v2 template/style/layout bridge |
| State | Visual state is expressible through retained component state and selector folding. | `UiComponentState`, `UiPainterStyleSelector`, v2 style resolver |
| Behavior | Mouse, keyboard, focus, popup, drag, or edit behavior is owned by runtime reducers/dispatch. | `state_reducer/*`, plan 01 dispatch, plan 03 text |
| Render | Runtime extraction and native painter paths can render the component without editor-specific code. | runtime surface/render path |
| Test | Focused tests and showcase rows cover the contract. | component catalog tests, Material Lab, later contract scripts |

## L1 Atom Matrix

| Component | Descriptor evidence | Asset evidence | Asset | Descriptor | Layout | State | Behavior | Render | Test | Next action |
|---|---|---|---|---|---|---|---|---|---|---|
| Label / Typography | `Typography` in `data_display.rs` | `material_typography.zui`, `workbench_section_title.zui` | Partial | OK | Partial | Partial | OK | Partial | Partial | Decide whether generic `Label` is an alias of `Typography` or add `workbench_label.zui`. |
| Icon / SvgIcon | `Icon`, `SvgIcon` in `data_display.rs`; plan 05 icon atlas | `material_icons.zui`, `material_material_icons.zui`, default `.icon.toml` seed pack | Partial | OK | Partial | Partial | OK | Partial | Partial | Add Workbench primitive or alias rule and connect atlas output to painter upload. |
| Button | `Button` in `button_inputs.rs` | `workbench_button.zui`, `material_buttons.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Finish keyboard activation and full selector matrix; button reducer now owns focus/hover/press. |
| IconButton | `IconButton` in `button_inputs.rs` | `workbench_icon_button.zui`, Material button assets | OK | OK | Partial | Partial | Partial | Partial | Partial | Reuse button-family reducer and validate icon slot/render parity. |
| FloatingActionButton | `FloatingActionButton` in `button_inputs.rs` | `material_floating_action_button.zui` | Partial | OK | Partial | Partial | Partial | Partial | Partial | Decide whether Workbench needs a primitive or Material showcase coverage is enough. |
| TextField / Field | `TextField`, `Input`, `TextareaAutosize` in `text_inputs.rs` | `workbench_field.zui`, `material_text_fields.zui`, `material_textarea_autosize.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Complete plan 03 edit chain, cursor/selection state, and multiline/autosize behavior. |
| SearchInput / SearchField | `SearchField` in `text_inputs.rs` | no dedicated Workbench primitive; Material text assets only | Missing | OK | Partial | Partial | Partial | Partial | Partial | Add `workbench_search_input.zui` or formalize Field+Icon composition. |
| NumberField | `NumberField` in `inputs.rs` | `workbench_axis_value_field.zui`, `material_number_field.zui` | Partial | OK | Partial | Partial | Partial | Partial | Partial | Numeric reducer owns step, large step, and clamp; remaining work is keyboard and drag capture policy. |
| Checkbox | `Checkbox` in `inputs.rs` | `workbench_checkbox.zui`, `material_checkboxes.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Add checked/indeterminate reducer ownership for click, label activation, and keyboard. |
| Radio | `Radio` in `inputs.rs` | `workbench_radio.zui`, `material_radio_buttons.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Extend selection reducer with radio-group focus and keyboard navigation. |
| Toggle / Switch | `Switch`, `ToggleButton` in `inputs.rs` | `workbench_toggle.zui`, `material_switches.zui`, `material_toggle_button.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Add checked-state reducer and disabled-aware activation policy. |
| Slider | `Slider` in `inputs.rs` | `workbench_slider.zui`, `material_slider.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Numeric reducer owns drag deltas; remaining work is track click, range mode, and keyboard step tests. |
| RangeSlider | no dedicated descriptor | no dedicated Workbench or Material range asset | Missing | Missing | Missing | Missing | Missing | Missing | Missing | Add descriptor, primitive asset, dual-thumb state, and collision/clamp rules. |
| Tab | `Tab` in `navigation_subcomponents.rs` | `workbench_tab.zui`, `material_tabs.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Add tab selection/focus reducer and arrow-key navigation. |
| TabStrip | `Tabs`/strip-level behavior not isolated as L1 primitive | `material_tabs.zui`; no `workbench_tab_strip.zui` | Partial | Partial | Partial | Partial | Partial | Partial | Partial | Split strip descriptor from tab item or map it to `TabStack` with explicit policy. |
| SegmentedControl | `ToggleButtonGroup` descriptor covers grouped selection | `workbench_segmented_control.zui`, `material_button_group.zui` | OK | Partial | Partial | Partial | Partial | Partial | Partial | Add first-class `SegmentedControl` descriptor or document group aliasing. |
| Dropdown trigger / Select | `Select`, `Dropdown`, `Autocomplete` in `selection_inputs.rs` | `workbench_dropdown.zui`, `material_selects.zui`, `material_autocomplete.zui` | OK | OK | Partial | Partial | Partial | Partial | Partial | Overlay reducer owns open/close and pointer anchor; remaining work is popup positioning, close policy, and keyboard navigation. |
| ProgressBar / Progress | `Progress` in `feedback.rs` | `material_progress.zui`; no Workbench primitive | Partial | Partial | Partial | Partial | OK | Partial | Partial | Decide `ProgressBar` alias/descriptor and add Workbench primitive if needed. |
| Badge / Tag | `Badge`, `Chip` in `data_display.rs` | `workbench_chip.zui`, `material_badges.zui`, `material_chips.zui` | Partial | OK | Partial | Partial | Partial | Partial | Partial | Separate passive tag, clickable chip, deletable chip, and badge-count behavior. |
| Divider | `Divider` in `data_display.rs` | `material_dividers.zui`; no Workbench primitive | Partial | OK | Partial | Partial | OK | Partial | Partial | Add Workbench divider primitive or document Material-only status. |
| Skeleton | `Skeleton` in `feedback.rs` | `material_skeleton.zui`; no Workbench primitive | Partial | OK | Partial | Partial | OK | Partial | Partial | Add animation policy in plan 07 and Workbench primitive if used by shell. |

## Current Implementation Implications

- No L1 atom has all seven gates at `OK` yet. `Button` and `IconButton` are the closest runtime-owned path because the descriptors now expose focus, hover, press, and commit, and `state_reducer/button.rs` owns the retained interaction flags.
- Material showcase assets cover more components than the Workbench primitive whitelist. For editor shell use, each Material-only atom needs either a Workbench primitive, a documented alias policy, or a governance exception.
- Reducer ownership is now split by behavior family: button interaction, generic interaction flags, disclosure, numeric drag, popup/overlay flags, selection, collection mutation, reference drop/location behavior, and world-space transform/surface metadata live under `state_reducer/`. Future slices should continue this pattern for text input, tab/choice behavior, and complex collection widgets.
- L2 collection groundwork is now represented in the same reducer structure: `state_reducer/windowing.rs` owns virtualized range aliases for `VirtualList`/`DataGrid` and page-window state for `Pagination`/`TablePagination`.
- Keyboard and focus completeness remains the dominant gap. Existing descriptors often advertise focus or value events, but Enter/Space activation, arrow-key movement, Escape close, and drag capture must be accepted only after runtime reducers and dispatch tests own the behavior.
