---
related_code:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/binding/update_report.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
implementation_files:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/superpowers/plans/2026-05-06-ui-lifecycle-reflection-reflector.md
  - user: 2026-05-06 continue UI lifecycle reflection reflector milestone
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
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

## Dirty Contract

Every accepted mutation marks the touched node dirty through `UiDirtyFlags`. The returned report carries the same structured dirty reason so callers can explain why a Widget Reflector edit requires layout, hit-test, render, text, or input refresh.

The binding report mirrors that invalidation through `UiBindingDirtyDomain`. If `UiSurface::mutate_property` adds render dirtiness while syncing runtime component state or pseudo-state styles, the binding report is refreshed so the report's retained invalidation and binding dirty domains stay consistent.

The legacy `state_flags.dirty` compatibility bit is now reserved for state changes that affect hit-test or input routing. Render-only changes must not set it, because `UiSurface::dirty_flags()` still treats that legacy bit as a conservative hit-test/input/render invalidation. This keeps paint-only state, Material metadata, and dispatch redraw effects on the render-only rebuild path instead of rebuilding the arranged tree or hit grid.

The current dirty mapping is intentionally conservative:

- `Collapsed` visibility marks layout, hit-test, render, and input dirty.
- Other visibility and input-affecting state marks hit-test, render, and input dirty.
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
