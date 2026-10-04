---
related_code:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/binding/update_report.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
implementation_files:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/superpowers/plans/2026-05-06-ui-lifecycle-reflection-reflector.md
  - user: 2026-05-06 continue UI lifecycle reflection reflector milestone
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - 2026-05-20: cargo test -p zircon_runtime --lib surface_property_mutation_marks_dirty_only_when_values_change --locked --jobs 1 --message-format short --color never (passed, 1 test)
  - 2026-05-20: cargo test -p zircon_runtime --lib accessibility_set_value --locked --jobs 1 --message-format short --color never (passed, 4 tests)
  - 2026-05-20 result-propagation: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - 2026-05-20 widget-result-propagation: rustfmt --edition 2021 --check zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs zircon_runtime/src/ui/surface/surface/default_interactions/range.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs zircon_runtime/src/ui/tests/widget_radio_behavior.rs zircon_runtime/src/ui/tests/widget_menu_behavior.rs zircon_runtime/src/ui/tests/widget_range_navigation.rs zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs (passed)
  - 2026-05-20 widget-result-propagation: cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing unused-method warning)
  - cargo test -p zircon_runtime --lib shared_core --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib event_routing --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib component_catalog --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo check -p zircon_runtime --lib --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib surface_dirty_text_edit_visual_metadata_stays_render_only --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_runtime --lib surface_dirty_domains --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 10 passed)
  - cargo check -p zircon_runtime --lib --jobs 1 (2026-05-13: passed)
  - cargo test -p zircon_runtime --lib surface_property_mutation_keeps_template_visibility_metadata_in_sync --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest workbench visibility refresh: passed, 1 passed)
  - cargo test -p zircon_runtime --lib ui_v2_surface_property_mutation_updates_runtime_style_baseline_metadata --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest runtime style baseline: passed, 1 passed)
  - cargo test -p zircon_runtime --lib ui_v2_surface_property_mutation_restyles_focused_pseudo_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 retained pseudo-state sync: passed, 1 passed)
  - 2026-06-16 Runtime 09 property visibility flag rename: rustfmt check; Python py_compile; direct ui_architecture_boundary_audit; standalone ui_architecture.rs 14/14; standalone plan-status status-output filters 2/2 (static passed, Cargo behavior filters pending)
doc_type: module-detail
---

# Runtime UI Property Mutation

`property_mutation.rs` is the runtime-owned mutation seam for reflected UI properties. It accepts a `UiPropertyMutationRequest`, mutates the retained `UiTree` node when the property is safe to edit at runtime, and returns a `UiPropertyMutationReport` that describes whether the request was accepted, unchanged, or rejected.

This module exists so editor/debug tooling can exercise the same retained surface state that runtime input dispatch uses. It does not rewrite authored `.ui.toml`, does not dispatch editor operations, and does not create a second reflection-only widget tree.

The report now also carries a `UiBindingUpdateReport`. That makes property mutation part of the M3 binding convergence path: accepted, unchanged, and rejected runtime writes all expose the same source/target/status/dirty DTO used by widget reducers, accessibility actions, and the future UI ECS bridge. Accepted writes that also change `UiSurfaceComponentStateStore` append a second component-state value update to the same report. Runtime-only request constructors let widget behavior and accessibility actions label the first binding update with `WidgetBehavior` or `AccessibilityAction` instead of losing that intent behind a generic runtime-state write.

## Accepted Properties

The first slice deliberately supports a narrow set of runtime-safe fields:

- `visibility`
- `enabled`
- `visible`
- `clickable`
- `hoverable`
- `focusable`
- `pressed`
- `checked`
- `input_policy`
- template metadata attributes when the node has `template_metadata`

Unknown property names fall through to `template_metadata.attributes`. This keeps authored-like metadata edits on the retained node and uses the same `UiValue::to_toml()` conversion that reflection snapshots use for authored attribute display. Nodes without template metadata reject unknown properties instead of inventing an attribute bag.

`visibility` and legacy `visible` are special because responsive layout can derive effective visibility from template metadata on every layout pass. Runtime mutation therefore keeps the retained field and any existing template metadata attribute synchronized. Without that synchronization, a panel changed from `collapsed` to `visible` can briefly update the retained node and then be reset to the stale authored value during `UiSurface::rebuild_dirty(...)`.

V2 runtime style adds one more source of staleness: pseudo-state style refresh rebuilds node metadata from a captured baseline. When an accepted runtime mutation changes a template metadata attribute, `UiSurface::mutate_property` now updates that runtime style baseline before component-state restyling runs. This preserves runtime writes such as `visibility = "visible"` when a component also has `:hover`, `:checked`, or other pseudo-state rules.

Retained pseudo-state keys also need to flow through `UiSurfaceComponentStateStore`, because v2 runtime style intentionally strips pseudo-state attributes such as `focused`, `hovered`, `active`, `popup_open`, and `selected` from plain metadata before re-applying active runtime states. `UiSurface::sync_component_state_from_property` mirrors both canonical names and aliases (`focus`/`focused`, `hover`/`hovered`, `active`/`pressed`, `open`/`popup_open`) into the component-state flags. That keeps editor preview controls visually focused or open after a dirty style refresh instead of showing a transient one-frame state.

## Dirty Contract

Every accepted mutation marks the touched node dirty through `UiDirtyFlags`. The returned report carries the same structured dirty reason so callers can explain why a Widget Reflector edit requires layout, hit-test, render, text, or input refresh.

The binding report mirrors that invalidation through `UiBindingDirtyDomain`. If `UiSurface::mutate_property` adds render dirtiness while syncing runtime component state or pseudo-state styles, the binding report is refreshed so the report's retained invalidation and binding dirty domains stay consistent.

The legacy `state_flags.dirty` compatibility bit is now reserved for state changes that affect hit-test or input routing. Render-only changes must not set it, because `UiSurface::dirty_flags()` still treats that legacy bit as a conservative hit-test/input/render invalidation. This keeps paint-only state, Material metadata, and dispatch redraw effects on the render-only rebuild path instead of rebuilding the arranged tree or hit grid.

Runtime 09 M1.2 records `runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending`: the visibility transition helper names the boolean passed into `UiVisibility::effective(...)` as `state_visible_flag`. The dirty behavior is unchanged; `runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt` only prevents the old local wording from returning.

The current dirty mapping is intentionally conservative:

- `Collapsed` visibility marks layout, hit-test, render, and input dirty.
- Other visibility and input-affecting state marks hit-test, render, and input dirty.
- A transition between collapsed and visible/hidden compares effective layout occupancy and marks layout dirty when occupancy changes, even when the requested final visibility would normally be a non-layout state.
- `pressed` and `checked` mark render dirty.
- text-like value metadata marks layout, render, and text dirty.
- text edit visual metadata such as caret, selection, and composition ranges marks render dirty only; the edited `value` property is the field that drives text layout and measurement.
- size/spacing metadata marks layout, hit-test, and render dirty.
- Material-style `layout_*` metadata marks layout, hit-test, and render dirty so retained layout metrics and reflected invalidation reasons stay aligned.
- other metadata marks render dirty.

Callers remain responsible for invoking `UiSurface::rebuild_dirty(root_size)` or a stronger rebuild after mutation. Mutation itself only changes retained state and dirty flags.

## Rejection Rules

Rejected requests return `UiPropertyMutationStatus::Rejected`, preserve the retained tree, and carry a human-readable message. Rejections currently cover invalid value kinds, invalid visibility/input policy tokens, and missing template metadata for arbitrary attributes. Missing node IDs return `UiTreeError::MissingNode` before a mutation report is created because there is no retained node that can own a reflected property report.

This is not a schema validator. Descriptor-level validation, binding side effects, editor undo/redo, and authored source persistence belong to higher runtime/editor layers.

## Binding Report Scope

This slice records the retained-property update plus the secondary component-state value update emitted by `UiSurface::sync_component_state_from_property`. Both updates share the final dirty domain union after component-state or pseudo-state style sync adds render dirtiness. Widget reducers and accessibility SetValue dispatch still call `mutate_property` as before, but their origin is preserved in the same report instead of only observing the retained-property result.

Default widget reducers now construct property mutations through `UiPropertyMutationRequest::widget_behavior`, while accessibility SetValue and range Increment/Decrement use `UiPropertyMutationRequest::accessibility_action` or the equivalent source-kind override. That preserves the origin in `UiBindingSourceKind` without changing the retained-tree mutation policy. SetValue and range adjustment attach the resulting binding report to `UiInputDispatchResult.binding_reports`; default widget actions attach their reports to pointer, navigation, keyboard, and shared input dispatch results. Accessibility Activate preserves the binding reports produced by the reused widget behavior path, while Accessibility ScrollTo records a runtime-state `scroll_offset` report outside retained-property mutation. That makes Button/Toggle/Radio/Menu/Range/TextInput, scrollbar scroll state, and accessibility action paths observable through the same report surface as component events.
