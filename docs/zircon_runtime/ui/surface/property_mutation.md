---
related_code:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
implementation_files:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/superpowers/plans/2026-05-06-ui-lifecycle-reflection-reflector.md
  - user: 2026-05-06 continue UI lifecycle reflection reflector milestone
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
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
