---
related_code:
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/tests/editing/reflected_command/mod.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
implementation_files:
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/tests/editing/reflected_command/mod.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - docs/crates/zircon_editor/core/editing/command.md
  - docs/crates/zircon_editor/scene/viewport/edit_mode_projection.md
tests:
  - zircon_editor/src/tests/editing/reflected_command/mod.rs
  - cargo test -p zircon_editor --lib reflected_editor_command --locked --jobs 1 --message-format short
doc_type: module-detail
---

# Editor State Snapshot Build

## Purpose

`editor_state_snapshot_build.rs` projects `EditorState` into the data snapshot consumed by editor chrome, retained UI, and remote workbench reflection. The snapshot remains editor-owned: it carries selection drafts, workbench panes, asset browser state, undo/redo flags, and viewport settings. Runtime `World` remains the authority for scene entities and component values.

The snapshot resolves editor selection through `Scene::editor_projection(...)` before marking scene rows or building inspector data. A stale editor-selected id can remain in controller state until normal selection synchronization runs, but it is not exposed as an active row or inspector target in the workbench snapshot.

## Reflection Contract

M10 routes plugin inspector property rows through runtime reflection when a schema is loaded. For each selected dynamic component, the snapshot builder resolves `Scene::reflect_schema(...)`, then reads component field values through `Scene::reflect_fields(...)`. Field labels, value type names, editability, plugin id, and display name come from `ReflectTypeRegistration` and `ReflectFieldInfo`, not from ad hoc descriptor/JSON property metadata.

Draft editor text is still stored in `EditorState::inspector_dynamic_fields` until `ApplyInspectorChanges` captures a command. The snapshot overlays that draft text onto the reflected property row by field id, preserving the same UI behavior while keeping the canonical field contract in runtime reflection.

Snapshot editability mirrors the generic reflected text parser used by inspector submission. Reflected bool, integer, unsigned, scalar, string, enum, resource, Vec2, Vec3, Vec4, quaternion, and entity values can be marked editable when their runtime schema field is editable. Lists, maps, raw JSON, and null values remain visible but protected until the editor has dedicated controls for those shapes.

## Unloaded Plugins

Serialized dynamic component JSON can exist before its plugin schema is loaded. In that case the snapshot falls back to protected JSON projection: properties remain visible for inspection, but `editable` is false and the diagnostic explains that serialized data stays protected until the plugin reloads. This preserves scene visibility without allowing editor mutations against an unknown schema.

## Validation

`zircon_editor/src/tests/editing/reflected_command/` covers both sides of the snapshot path through its snapshot projection and inspector mutation tests:

- loaded dynamic schemas produce reflected plugin property rows with schema value kinds and editability,
- draft values override reflected readback only in the snapshot,
- vector and entity reflected rows remain editable and preserve draft text before command capture,
- unloaded dynamic JSON remains visible and non-editable,
- stale editor selection does not leak into scene rows or inspector snapshots,
- inspector submission later writes through `EditorCommand::set_reflected_scene_field(...)`.

The focused Cargo command is currently blocked by the shared workspace Cargo/Rust queue in this checkout, so fresh pass/fail evidence must be collected after the queue clears.

## Read-only rotation projection

`InspectorSnapshot.rotation_degrees` is an optional local XYZ Euler display value. Edit snapshots read the selected entity's real `Scene::local_transform` quaternion; Play snapshots read the matching runtime `LocalTransform.rotation` inspection value: the current World producer uses `ReflectedValue::Vec4`; Quaternion values are also accepted. Both carry the same quaternion coefficients and use the shared conversion. The shared conversion normalizes finite nonzero quaternions, uses the engine `EulerRot::XYZ` convention in radians, converts to degrees, and formats two decimal places. Quaternion storage and world authority remain unchanged. Missing or invalid rotation is unavailable, never synthesized as identity. Preview objects lacking a quaternion carry `None`.

The authored Inspector bridge writes these three values into the existing rotation controls. Their disabled/noneditable flags and absent commit routes remain intact because runtime reflection still exposes rotation as read-only. Translation and scale keep their existing transaction routes. Rotation presentation depends on the shared retained-surface attribute precedence repair; immutable authored sample values must not replace the live surface. Full component coverage is separate: the component section still selects the first plugin-owned component snapshot and is not a complete view of built-in Camera/Mesh data.
