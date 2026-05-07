---
related_code:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
implementation_files:
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/superpowers/plans/2026-05-06-ui-lifecycle-reflection-reflector.md
  - user: 2026-05-06 continue UI lifecycle reflection reflector milestone
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
  - cargo test -p zircon_runtime --lib shared_core --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib event_routing --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib component_catalog --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo check -p zircon_runtime --lib --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
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

Every accepted mutation marks the touched node dirty through `UiDirtyFlags` and sets the legacy `state_flags.dirty` compatibility bit. The dirty flag is part of the returned report so callers can explain why a Widget Reflector edit requires layout, hit-test, render, text, or input refresh.

The current dirty mapping is intentionally conservative:

- `Collapsed` visibility marks layout, hit-test, render, and input dirty.
- Other visibility and input-affecting state marks hit-test, render, and input dirty.
- `pressed` and `checked` mark render dirty.
- text-like metadata marks layout, render, and text dirty.
- size/spacing metadata marks layout, hit-test, and render dirty.
- Material-style `layout_*` metadata marks layout, hit-test, and render dirty so retained layout metrics and reflected invalidation reasons stay aligned.
- other metadata marks render dirty.

Callers remain responsible for invoking `UiSurface::rebuild_dirty(root_size)` or a stronger rebuild after mutation. Mutation itself only changes retained state and dirty flags.

## Rejection Rules

Rejected requests return `UiPropertyMutationStatus::Rejected`, preserve the retained tree, and carry a human-readable message. Rejections currently cover invalid value kinds, invalid visibility/input policy tokens, and missing template metadata for arbitrary attributes. Missing node IDs return `UiTreeError::MissingNode` before a mutation report is created because there is no retained node that can own a reflected property report.

This is not a schema validator. Descriptor-level validation, binding side effects, editor undo/redo, and authored source persistence belong to higher runtime/editor layers.
