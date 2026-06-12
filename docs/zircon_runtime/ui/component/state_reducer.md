---
related_code:
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
  - zircon_runtime/src/ui/component/catalog/material_foundation/button_inputs.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/tests/component_catalog/complex_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/button.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/windowing.rs
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
  - zircon_runtime/src/ui/component/catalog/material_foundation/button_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/complex_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/button.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/windowing.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - user: 2026-05-07 single-select selection lists must keep one selected item or none
tests:
  - zircon_runtime/src/ui/tests/component_catalog/component_state/button.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\catalog\material_foundation\button_inputs.rs zircon_runtime\src\ui\tests\component_catalog\component_state.rs zircon_runtime\src\ui\tests\component_catalog\component_state\button.rs zircon_runtime\src\ui\tests\component_catalog\material_foundation\button_inputs.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-button-family-0612-coremin --message-format short --color never (passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\button.rs zircon_runtime\src\ui\component\state_reducer\selection.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-selection-family-0612-coremin --message-format short --color never (passed with existing warnings)
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
  - cargo test -p zircon_runtime --lib material_button_family --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-button-family-0612-coremin --message-format short --color never -- --test-threads=1 --nocapture (timed out after 904s during lib-test compile/link with no Rust diagnostics; matching cargo/rustc processes were stopped or had exited)
  - cargo check -p zircon_runtime --lib (passed with existing warnings)
  - cargo test -p zircon_runtime component_state_single_selection -- --nocapture (blocked by pre-existing test compile error: zircon_runtime/src/ui/tests/asset_resource_refs.rs references missing RESOURCE_DEPENDENCY_LAYOUT)
doc_type: module-detail
---

# Component State Reducer

`zircon_runtime::ui::component::state_reducer` applies component-level UI events to `UiComponentState`. The public entry point remains `apply_component_event(...)`, while family-specific behavior lives in child modules under `state_reducer/`.

## Family Dispatch

`state_reducer/button.rs` is the first family reducer extracted from the monolithic entry function for plan 06 M1.S2. It recognizes `Button`, `IconButton`, `FloatingActionButton`, and `ButtonBase` by descriptor id or role, then owns the transient button interaction flags:

- `Focus` writes `state.flags.focused`.
- `Hover` writes `state.flags.hovered`.
- `Press` writes `state.flags.pressed`.

Other button-family events fall back to the generic reducer. This preserves the existing `Commit { property, value }` path, including descriptor-based value validation and reference-source cleanup, instead of duplicating generic property mutation in the button module.

The Material foundation descriptors for `Button`, `IconButton`, and `FloatingActionButton` now expose `Focus`, `Hover`, `Press`, and `Commit`. `ButtonGroup` stays structural; child buttons own interaction state and commit routing.

`state_reducer/interaction.rs` owns generic retained interaction flags shared by non-button rows, drag targets, and future composed controls: focus, hover, press, drag begin/end, drop hover, and active drag target.

`state_reducer/disclosure.rs` owns the retained disclosure flag used by `ToggleExpanded`; it updates both `state.flags.expanded` and the `expanded` value property so selector state and data binding stay aligned.

`state_reducer/selection.rs` owns the existing `SelectOption` behavior. Moving this family out of the entry file is a structural split: single-select, multi-select, flags, disabled-option rejection, stale-value conversion, and `state.flags.selected` semantics are unchanged.

`state_reducer/collection.rs` owns the existing ArrayField/MapField mutation family: add/set/remove/move array elements, add/set/rename/remove map entries, row-level validation errors, and reference-source cleanup for replaced collection values. Moving this family reduced the dispatcher file to a thin routing layer for collection events while keeping existing `ArrayIndexOutOfBounds`, `DuplicateMapKey`, and `MissingMapKey` behavior unchanged.

`state_reducer/numeric.rs` owns numeric drag math for `DragDelta` and `LargeDragDelta`. It validates that the target property is numeric, reads the current/default value, applies either `step` or `large_step`, clamps through the descriptor/state `min` and `max` settings, and writes the normalized `Int` or `Float` value through the shared `set_value(...)` helper so reference-source cleanup remains consistent.

`state_reducer/overlay.rs` owns the popup flag and pointer-anchor behavior used by selection popups, context menus, and later overlay surfaces. `OpenPopup` and `ClosePopup` only mutate `state.flags.popup_open`; `OpenPopupAt` also records `popup_anchor_x` and `popup_anchor_y` through the shared value path so stale reference metadata is cleared if those property names were ever reused.

`state_reducer/reference.rs` owns the retained reference field behavior: accepted asset drops write `UiValue::AssetRef`, accepted scene instance/object drops write `UiValue::InstanceRef`, optional drag source metadata is retained per property, sourceless accepted drops clear stale source metadata, `ClearReference` writes `UiValue::Null`, and locate/open actions validate that a non-empty reference exists. Rejected drops still leave the previous value and source metadata untouched while setting validation feedback.

`state_reducer/windowing.rs` owns virtualized visible-range and page-window math. `SetVisibleRange` derives canonical `total_count`, viewport bounds, requested overscan bounds, and scroll offsets from both Zircon and MUI/react-window alias props. `SetPage` clamps invalid page sizes and out-of-range page indices, then stores `page_size`, `page_count`, `page_index`, `page_start`, `page_end`, and `empty`.

`state_reducer/world.rs` owns world-space UI metadata mutation. `SetWorldTransform` validates positive scale before writing `world_position`, `world_rotation`, and `world_scale`; `SetWorldSurface` validates positive size, clamps `pixels_per_meter` through the descriptor range, and writes billboard, depth-test, render-order, and camera-target metadata for host world-space UI integration.

## Selection Semantics

`apply_selection(...)` resolves selection mode from the component descriptor and current state:

- A property whose descriptor declares `UiValueKind::Flags` is a flag set and stores `UiValue::Flags(Vec<String>)`.
- A component whose `multiple` setting is true stores selected values in `UiValue::Array`.
- Every other selection is single-select and stores either `UiValue::Enum(option_id)` or `UiValue::Null`.

The mode decision is made before inspecting any stale value currently stored on the property. This matters because old state, partially migrated data, or previous bugs can leave a single-select control with `UiValue::Flags` or `UiValue::Array`. A single-select event must not preserve that stale container and append into it. It must replace the value with the one selected enum, or clear it to null when the option is unselected.

## Stale Value Conversion

For flag properties, `selection_flags_value(...)` removes the previous value and converts known option-shaped values into a clean `Vec<String>`. Existing `Flags` values are preserved, arrays keep enum/string entries, and a non-empty enum/string becomes a one-item flag vector. Other values normalize to an empty flag list.

For multi-select properties, `selection_array_value_mut(...)` keeps the existing array path and converts a previous scalar enum/string into a one-item array before adding or removing the requested option.

For single-select properties, no previous container is reused. Selecting an option writes `UiValue::Enum(option_id)`. Unselecting writes `UiValue::Null`. This keeps the public invariant simple: a single-select value is unique or absent, even when the previous state was malformed.

`state.flags.selected` is still updated from the event's selected flag after the value mutation. That flag reflects the most recent event outcome; it is not the source of truth for the selected option list.

## Focused Regression Coverage

`material_button_family_applies_interaction_flags_through_public_reducer` initializes Material `Button`, `IconButton`, and `FloatingActionButton` descriptors through `UiComponentDescriptorRegistry::material_editor_foundation()`, then applies `Focus`, `Hover`, and `Press` through `UiComponentStateRuntimeExt::apply_event(...)`. It asserts the public reducer path accepts and clears the retained interaction flags.

`material_button_family_preserves_commit_value_delivery` keeps the existing button `Commit` fallback path covered by applying `Commit { property: "activated", value: true }` to `Button` and verifying the generic value mutation remains intact after the family split.

`material_selection_popups_update_retained_popup_flags_through_public_reducer` covers `Select` and `Autocomplete` popup open/close flags through the Material foundation descriptors, and `popup_anchor_events_record_pointer_anchor_through_public_reducer` covers the anchored popup path used by context menus.

`material_pagination_events_update_retained_page_window` and `material_pagination_window_handles_empty_and_invalid_page_size` cover the `SetPage` route used by `Pagination` and `TablePagination`, including page-size normalization, clamping, and empty data.

`component_state_single_selection_replaces_stale_flags_value` initializes a dropdown with a stale `UiValue::Flags(["runtime", "debug"])`, selects `"editor"`, and expects `UiValue::Enum("editor")`.

`component_state_single_selection_clears_to_none_when_unselected` initializes a combo box with `UiValue::Enum("runtime")`, unselects `"runtime"`, and expects `UiValue::Null`.

Scoped `cargo check -p zircon_runtime --lib --no-default-features --features core-min` passes after the current family splits, with existing warning noise only. Focused lib-test execution remains expensive in this workspace and previously timed out while building/linking the runtime lib-test target before the filtered button tests could run.
