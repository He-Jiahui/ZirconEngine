---
related_code:
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
implementation_files:
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
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

# Runtime UI Reflection Snapshot

`reflection_snapshot.rs` derives a read-only `UiReflectorSnapshot` from `UiSurface`. It is the runtime producer for Widget Reflector-style inspection and is intentionally built from the retained `UiTree`, arranged tree, hit-test query result, and focus/capture/hover state already owned by the surface.

The snapshot is a view of runtime truth. It does not rebuild layout, does not create a private hit index, and does not own a parallel widget tree.

## Node Projection

Each retained node becomes a `UiReflectorNode` keyed by `UiNodeId`. The node carries the data needed by editor/debug consumers without requiring them to query runtime internals:

- tree path, parent, children, class name, and display name
- declared and effective visibility
- state flags and input policy
- z index, paint order, frame, clip frame, and clip behavior
- dirty flags
- reflected properties
- action descriptors derived from template bindings
- focused, hovered, captured, and pressed state
- source template path metadata

Frames prefer arranged output when present and fall back to retained layout cache when the node has not entered the arranged tree. Effective visibility follows arranged visibility when available and otherwise uses `UiTreeNode::effective_visibility()`.

## Lifecycle Heuristic

The snapshot maps retained runtime state into `UiWidgetLifecycleState` for inspection:

- invisible legacy state or collapsed effective visibility becomes `Detached`
- focusable or pointer-capable nodes become `Interactive`
- render-visible nodes become `Visible`
- arranged non-render nodes become `Arranged`
- template metadata marks `PropertiesSynchronized`
- otherwise the node is `Constructed`

This lifecycle is diagnostic vocabulary, not a new runtime scheduler. Runtime behavior remains governed by retained tree state, layout, dispatch, render extraction, and dirty rebuilds.

## Properties And Actions

Reflected properties mirror the runtime mutation seam. System/runtime fields such as `visibility`, `input_policy`, `enabled`, `clickable`, and `checked` are writable in the snapshot because `UiSurface::mutate_property(...)` can apply them. Template metadata attributes are reflected as authored values and use the same dirty classification as runtime mutation.

Actions are projected from `template_metadata.bindings`. The `binding_symbol` prefers an explicit action name, then the action route, then the binding-level route, and finally the binding id. That keeps authored `.ui.toml` route bindings visible in the Reflector before all actions are backed by remote-callable operation metadata.

## Hit Context

When a `UiHitTestQuery` is supplied, the snapshot records the query point, top hit node, and stacked hit nodes from the surface hit grid. `UiHitTestResult` does not currently expose rejected candidate reasons, so `UiReflectorHitContext.rejected` is deliberately empty until the lower hit-test diagnostics surface that data.
