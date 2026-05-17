---
related_code:
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/local_transform.rs
  - zircon_runtime/src/scene/reflect/fixed/name.rs
  - zircon_runtime_interface/src/reflect/object_address.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/component.rs
  - dev/bevy/crates/bevy_ecs/src/world/reflect.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
implementation_files:
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-grade ECS/Reflect/Scene/Transform roadmap execution
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - docs/zircon_runtime/scene/reflect.md
  - docs/zircon_editor/scene/viewport/edit_mode_projection.md
tests:
  - zircon_editor/src/tests/editing/reflected_command.rs
  - cargo test -p zircon_editor --lib reflected_editor_command --locked --jobs 1 --message-format short
doc_type: module-detail
---

# Editor Command Reflection Path

## Purpose

`zircon_editor::core::editing::command` owns the undoable mutation boundary for scene editing. Runtime `World` remains the authority for scene state, while editor commands capture enough before/after data to apply, undo, and redo user actions.

The M10 reflection command slice adds `SetReflectedSceneFieldCommand` and removes the editor undo/redo mutation path that previously wrote through `ComponentPropertyPath`. Editor history now mutates component fields through the runtime reflection facade instead of routing plugin fields through legacy property strings or the compact fixed-node update path. This aligns inspector editing with the runtime projection work: the viewport now reads inspector rows from reflected runtime metadata, and inspector submission writes the same reflected field contract back into the scene for fixed and plugin-owned fields.

`ComponentPropertyPath` can still appear in animation and asset authoring code as asset data, but it is no longer an editor scene mutation command. Runtime scene state that is changed through the inspector must pass through reflected component addresses and reflected field names.

## Bevy Source Basis

The Bevy reference for this slice is the reflection bridge between ECS components, type registration, and world access:

- `dev/bevy/crates/bevy_reflect/src/type_registry.rs:19` defines `TypeRegistry` as the central store for reflected type information. Its registration APIs around `register` and `GetTypeRegistration` make reflected metadata discoverable from a runtime registry.
- `dev/bevy/crates/bevy_ecs/src/reflect/component.rs:151` defines `ReflectComponent` operations such as insert, apply, reflect, reflect_mut, and copy for component values once their type metadata exists.
- `dev/bevy/crates/bevy_ecs/src/world/reflect.rs:13` exposes world-level reflected component reads through `World::get_reflect`, and the mutable variant starts at `dev/bevy/crates/bevy_ecs/src/world/reflect.rs:117`.

Zircon intentionally mirrors the architectural shape rather than copying Bevy's exact API. Bevy uses Rust `TypeId`, derived `Reflect`, and `AppTypeRegistry`; Zircon uses serializable `type_path` strings, `ReflectObjectAddress`, `ReflectReadRequest`, `ReflectWriteRequest`, and explicit fixed/dynamic adapters. The important convergence point is that editor code no longer needs bespoke knowledge of every component storage layout to edit a field.

## Command Model

`SetReflectedSceneFieldCommand` stores:

- target entity id,
- reflected component type path,
- reflected field name,
- `ReflectedValue` before the edit,
- `ReflectedValue` after the edit,
- selection before and after the command.

Capture is eager. `EditorCommand::set_reflected_scene_field(...)` first calls `Scene::reflect_read(...)` to snapshot the current field value. If the value is unchanged, it returns `Ok(None)` and does not create history noise. Otherwise it calls `Scene::reflect_write(...)`; the normalized value returned by `ReflectWriteResponse` becomes the command's `after` value.

Apply and undo both call the same runtime reflection write helper. This keeps fixed components, plugin-owned dynamic JSON components, editability checks, field type validation, and missing entity/component errors centralized in `zircon_runtime::scene::reflect`.

## Inspector Flow

`EditorState::apply_inspector_changes` turns the fixed node form into reflected updates before command capture:

1. `name_field` becomes `zircon_runtime::scene::components::Name.value`.
2. `parent_field` becomes `zircon_runtime::scene::components::Hierarchy.parent`.
3. `transform_fields` become `zircon_runtime::scene::components::LocalTransform.translation`.
4. `inspector_dynamic_fields` still contains reflected ids emitted by runtime projection, such as `weather.Component.CloudLayer.coverage`.
5. The editor splits each reflected id into `component_type_path` and `field_name`, creates `EditorCommand::set_reflected_scene_field(...)`, batches the captured commands, and pushes the batch to editor history.

The fixed form keeps its dedicated validation for trimmed non-empty names, parent entity parsing, and Vec3 translation parsing. The generic reflected component text parser currently accepts bool, signed integer, unsigned integer, scalar, string, enum, resource, Vec2, Vec3, Vec4, quaternion, and entity reflected values. Vector-like values accept comma-separated or whitespace-separated finite numbers with optional `[]` or `()` wrappers, and entity values accept an entity id or `none`/`null`. Maps, lists, raw JSON, and null values remain unsupported by the generic component drawer until the editor has dedicated controls for those shapes.

The UI adapter editability gate uses the same reflected contract before it accepts draft field changes. `EditorState::can_edit_dynamic_component_field` resolves the field id into a reflected component address, checks `Scene::reflect_schema(...)` for an editor-visible editable field, and then confirms the selected entity can read that reflected field. A loaded dynamic JSON blob without a registered reflected schema is visible as protected data but is not accepted for generic inspector mutation.

The workbench data snapshot uses the same reflected source for plugin component property rows. When a schema is loaded, `EditorState::snapshot_with_component_drawers` projects plugin fields from `Scene::reflect_schema(...)` and `Scene::reflect_fields(...)`, while draft text remains editor-owned until command capture. If a dynamic JSON component has no loaded schema, the snapshot falls back to protected read-only JSON rows and emits the existing unloaded-plugin diagnostic.

## Failure Behavior

Read-only reflected fields fail before a command is captured because runtime `reflect_write` returns `ReflectError::NonEditableField`. Unknown fields, missing components, unknown type paths, and type mismatches surface as command capture errors and leave scene state unchanged.

Undo returns the previous selection recorded at capture time; redo/apply selects the edited entity. This preserves the editor history selection behavior that the old scene property command provided, while keeping all inspector-owned field mutation on the reflected command path.

## Validation

`zircon_editor/src/tests/editing/reflected_command.rs` covers the reflected command surface:

- fixed component editing through the reflected `Name.value` field,
- dynamic plugin JSON component editing through `weather.Component.CloudLayer.coverage`,
- read-only dynamic field rejection through `weather.Component.CloudLayer.label`,
- reflected schema/read editability gating for editable, read-only, unknown, and unloaded-schema dynamic fields,
- reflected workbench snapshot property projection for loaded plugin schemas and protected unloaded JSON data,
- end-to-end inspector submission, undo, and redo through `EditorState::apply_inspector_changes`, including fixed `Name.value`, `Hierarchy.parent`, `LocalTransform.translation`, generic `LocalTransform.scale` Vec3 text, generic entity field text, and dynamic plugin fields in reflected command batches.

The focused validation command for this slice is:

```powershell
cargo test -p zircon_editor --lib reflected_editor_command --locked --jobs 1 --message-format short
```

Earlier local evidence for the reflected command slice: the focused command passed with 4 tests, 0 failures, and 1342 filtered tests. After the fixed-form cutover expanded the same test module and the old editor scene property command was removed, local non-Cargo checks passed, but fresh Cargo output is still pending because the shared checkout's active Cargo/Rust queue kept the focused command queued with empty stdout/stderr.

The wider `zircon_editor` check is currently expected to stop in the active rendering parity lane until `zircon_runtime/src/scene/world/render.rs` is updated for the new `PostProcessExtract` fields. This command module does not own that rendering initializer.

## Follow-up

The editor command surface no longer has a legacy `SetScenePropertyCommand` entry point. Remaining M10 work should keep editor-facing component mutation on `EditorCommand::set_reflected_scene_field(...)`, expand typed controls for reflected value shapes that still need dedicated UI, and avoid reintroducing direct scene property writes for inspector data.
