---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document.rs
  - zircon_runtime/src/scene/dynamic_scene/entity.rs
  - zircon_runtime/src/scene/dynamic_scene/error.rs
  - zircon_runtime/src/scene/dynamic_scene/patch.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/value.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/reflect_resource.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
implementation_files:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/document.rs
  - zircon_runtime/src/scene/dynamic_scene/entity.rs
  - zircon_runtime/src/scene/dynamic_scene/error.rs
  - zircon_runtime/src/scene/dynamic_scene/patch.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/value.rs
plan_sources:
  - user: 2026-05-16 Bevy-grade ECS/reflect/scene/transform completion request
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - dev/bevy/crates/bevy_scene/src/scene.rs
tests:
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib scene::tests::dynamic_scene --locked --jobs 1 --message-format short
doc_type: module-detail
---

# Dynamic Scene Serialization

`zircon_runtime::scene::dynamic_scene` is the runtime-side dynamic scene layer for milestone M9 of the Bevy-grade ECS/reflect/scene plan. It sits above the fixed `World` scene records and the reflection registry, giving Zircon a serializable scene artifact that can be captured from one world and spawned into another with entity remapping.

The design follows the shape of Bevy's `bevy_scene` APIs rather than replacing Zircon's existing project save/load path. In Bevy, `DynamicScene` is reflection-driven scene data, `Scene` is a concrete world snapshot, and `ScenePatch` applies reflected scene data onto a world. Zircon keeps its fixed `NodeRecord` world boundary intact, then adds reflected component/resource payloads around it.

## Public Types

- `DynamicScene` is a versioned serializable snapshot. It owns `entities` and `resources`, and exposes `from_world` plus `spawn_into`.
- `DynamicEntity` stores the source entity id, its fixed `NodeRecord`, and reflected components found through the `TypeRegistry`.
- `DynamicComponent` stores a component type path, whether it is plugin-owned, and its serializable reflected fields.
- `DynamicResource` stores a resource type path and its serializable reflected fields.
- `EntityRemap` records old scene ids to target world ids. It preserves ids when available and allocates the next free id when the target world already contains a source id.
- `ScenePatch` wraps a `DynamicScene` and applies it to a target `World`.
- `DynamicSceneError` reports format, duplicate id, missing parent, entity id exhaustion, world mutation, reflected value conversion, and reflection errors.

## Versioned Documents

`DynamicScene::from_versioned_json` is the migration boundary for runtime scene JSON. It accepts both the new `DynamicScene` document shape and the current legacy project document shape that stores a serialized `World` under `world`.

When the parser sees a top-level `world` field, it deserializes the legacy world and immediately exports it through `DynamicScene::from_world`. New documents are parsed directly as `DynamicScene` and checked against `DYNAMIC_SCENE_FORMAT_VERSION`.

`DynamicScene::to_versioned_json_pretty` writes the new document shape. This keeps migration one-way: legacy files can load into the dynamic scene model, while new saves should be emitted as reflected dynamic scene documents instead of preserving the old `ProjectDocument { world }` wrapper.

## Capture Flow

`DynamicScene::from_world` reads the sorted scene nodes from `World::node_records`, expands them back into `NodeRecord` values, and fills the fixed fields that `SceneNode` does not expose directly by asking the world for active state, render layer mask, and mobility.

For every entity, the dynamic scene exporter iterates `world.type_registry().iter()` and keeps registrations that are:

- components,
- marked serializable,
- backed by a `ReflectComponent` adapter,
- present on the entity.

The adapter's `read_fields` result is filtered to schema fields marked serializable. This keeps the serialized payload bound to the reflection schema instead of dumping arbitrary adapter output.

Resources use the same rule with `ReflectResource`: serializable resource registrations with an adapter are captured only when the adapter reports that the resource exists in the source world.

## Spawn And Patch Flow

`DynamicScene::spawn_into` performs four ordered steps:

1. Validate the format version and duplicate source ids.
2. Build an `EntityRemap` by preserving each source id when the target world does not already contain it, otherwise walking to the next free id that is not already reserved by this scene spawn.
3. Insert remapped `NodeRecord` values into the target world, remapping parent ids and joint connected entities.
4. Apply reflected components and resources.

Plugin-owned dynamic components are applied as complete JSON objects built from all serialized reflected fields. This intentionally preserves read-only plugin fields such as labels or authored metadata, because attaching a full dynamic component is a scene instantiation operation, not an editor property write.

Non-plugin fixed components and reflected resources are applied through their reflection adapters. Only fields that still exist in the target schema and are both serializable and editable are written. Read-only fixed data is expected to be carried by `NodeRecord` when it belongs to the core scene model.

## Entity Reference Remapping

Entity ids inside reflected values are remapped when applying a scene:

- `ReflectedValue::Entity(Some(id))` maps through `EntityRemap`.
- nested `List` and `Map` values are remapped recursively.
- `Json` values recursively remap objects shaped as `{ "entity": <id-or-null> }`.
- resource references are left unchanged.

This is the first step toward Bevy-style scene instancing semantics: serialized entity references follow the instantiated entity ids instead of pointing back at the original source world.

## Current Boundaries

This module does not yet serialize component type descriptors, create missing reflected resources, or integrate with asset hot-reload and asynchronous scene spawning. Those remain follow-up milestones after M9. The current layer is intentionally scoped to runtime reflection data that the target world already knows how to interpret.

## Validation

M9 added `zircon_runtime/src/scene/tests/dynamic_scene.rs` with coverage for:

- serializing a world with a plugin-owned reflected dynamic component,
- deserializing the dynamic scene with `serde_json`,
- spawning into a target world where source entity ids collide and verifying the remap,
- preserving dynamic component JSON including read-only fields,
- applying a `ScenePatch` that writes reflected resource fields through a resource adapter.
- migrating a legacy `ProjectDocument { world }` JSON payload into the new versioned dynamic scene shape, then reloading the new JSON document.

Validation commands for this milestone are:

```powershell
cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short
cargo test -p zircon_runtime --lib scene::tests::dynamic_scene --locked --jobs 1 --message-format short
```

Latest local evidence:

- `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short` passed on Windows after M9 implementation.
- After the versioned legacy migration slice, the same scoped check reached crate compilation but failed in an unrelated active asset/texture edit: `zircon_runtime/src/asset/assets/texture/texture_asset.rs:7:49` imports `TextureArrayLayout` from `asset::assets::texture`, where that symbol is not currently exported. This file is owned by active asset-image/texture sessions, so M9 did not modify it.
- After the active asset/texture export issue was resolved by its owning session, `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short` passed again.
- `cargo test -p zircon_runtime --lib scene::tests::dynamic_scene --locked --jobs 1 --message-format short` passed: 3 passed, 0 failed, 1434 filtered out. Earlier focused test attempts did not reach Zircon test execution because they stopped during dependency compilation while other workspace Cargo validations were active.
