---
related_code:
  - zircon_editor/src/ui/workbench/reflection/widget_reflector.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/tests/workbench/reflection/widget_reflector.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
implementation_files:
  - zircon_editor/src/ui/workbench/reflection/widget_reflector.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/superpowers/plans/2026-05-06-ui-lifecycle-reflection-reflector.md
  - user: 2026-05-06 continue UI lifecycle reflection reflector milestone
tests:
  - zircon_editor/src/tests/workbench/reflection/widget_reflector.rs
  - cargo test -p zircon_editor --lib workbench_reflection --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo check -p zircon_editor --lib --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
doc_type: module-detail
---

# Workbench Widget Reflector Model

`widget_reflector.rs` is the editor-side read model for runtime-produced `UiReflectorSnapshot` data. It gives workbench tooling a tree-row projection, a selected-node details view, the original snapshot export, and optional hit context without owning runtime UI state.

The model is intentionally small: it stores one snapshot and one local `selected_node`. Selection affects only the editor inspection view. It does not mutate the runtime `UiSurface`, does not rebuild a hit grid, and does not maintain a second retained widget tree.

## Rows

`WorkbenchWidgetReflectorModel::rows()` walks snapshot roots first and then appends any unvisited nodes. This preserves the runtime tree hierarchy while still making orphaned or diagnostic-only nodes visible to the inspector.

Each `WorkbenchWidgetReflectorRow` contains the stable node id/path, parent, depth, display metadata, lifecycle, visible/enabled state, dirty marker, and focus/hover/capture state. The row is a UI projection only; detailed properties stay on the selected node.

## Selection

`set_selected_node(...)` validates the requested id against the snapshot before changing selection. Missing nodes return `WorkbenchWidgetReflectorError::MissingNode` and leave the previous selection intact.

`selected()` returns the selected `UiReflectorNode` plus borrowed reflected properties. The properties are borrowed from the snapshot so the editor cannot diverge from the runtime snapshot by editing local copies.

## Boundary

This module is the first editor Widget Reflector consumer surface for the lifecycle/reflection milestone. Runtime owns the source snapshot and property mutation seam; the workbench model owns only local presentation state. Future UI panels should continue to consume `UiReflectorSnapshot` through this read model or another equally read-only adapter instead of querying Slint widgets or runtime internals directly.
