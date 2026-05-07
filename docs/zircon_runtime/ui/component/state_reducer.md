---
related_code:
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
implementation_files:
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
plan_sources:
  - user: 2026-05-07 single-select selection lists must keep one selected item or none
tests:
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - cargo check -p zircon_runtime --lib (passed with existing warnings)
  - cargo test -p zircon_runtime component_state_single_selection -- --nocapture (blocked by pre-existing test compile error: zircon_runtime/src/ui/tests/asset_resource_refs.rs references missing RESOURCE_DEPENDENCY_LAYOUT)
doc_type: module-detail
---

# Component State Reducer

`zircon_runtime::ui::component::state_reducer` applies component-level UI events to `UiComponentState`. Selection events are one of the important shared paths because editor controls, runtime UI assets, and host-projected controls all depend on a consistent interpretation of `SelectOption`.

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

`component_state_single_selection_replaces_stale_flags_value` initializes a dropdown with a stale `UiValue::Flags(["runtime", "debug"])`, selects `"editor"`, and expects `UiValue::Enum("editor")`.

`component_state_single_selection_clears_to_none_when_unselected` initializes a combo box with `UiValue::Enum("runtime")`, unselects `"runtime"`, and expects `UiValue::Null`.

`cargo check -p zircon_runtime --lib` passes after this change. The focused runtime test command is currently blocked before execution by an unrelated existing test compile error in `asset_resource_refs.rs`, where `RESOURCE_DEPENDENCY_LAYOUT` is referenced but not defined.
