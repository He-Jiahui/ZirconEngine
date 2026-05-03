---
related_code:
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
implementation_files:
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --color never -vv
doc_type: module-detail
---

# Navigation Framework Contracts

## Purpose

`zircon_runtime::core::framework::navigation` is the neutral contract layer for 3D navigation. It does not own Recast state, editor panels, or scene-authoring behavior. Instead, it defines the data shapes that runtime plugins, editor extensions, baked assets, and scene dynamic components agree on.

The module follows the navigation plugin plan: Unity-style authoring components are represented as dynamic component descriptors, while Unreal/Recast-style runtime behavior is routed through a `NavigationManager` trait and baked `.znavmesh` assets.

## Related Files

The main DTOs live in `zircon_runtime/src/core/framework/navigation/mod.rs`. Baked data lives in `zircon_runtime/src/asset/assets/navigation.rs` and is exposed through `ImportedAsset::{NavMesh, NavigationSettings}` plus `ResourceKind::{NavMesh, NavigationSettings}`. Dynamic component property JSON conversion is extended in `zircon_runtime/src/scene/world/dynamic_components.rs`.

## Behavior Model

The framework defines five fixed dynamic component type ids:

- `navigation.Component.NavMeshSurface`
- `navigation.Component.NavMeshModifier`
- `navigation.Component.NavMeshAgent`
- `navigation.Component.NavMeshObstacle`
- `navigation.Component.NavMeshOffMeshLink`

The default humanoid agent matches the plan values: radius `0.5`, height `2.0`, climb `0.4`, slope `45`, speed `3.5`, acceleration `8.0`, angular speed `360`, and stopping distance `0.1`. Areas reserve `0` for `not_walkable`, `1` for `walkable`, `2` for `jump`, and `3..63` for custom areas.

`NavMeshAsset` stores deterministic baked data: vertices, indices, polygons, tiles, off-mesh links, agent type, a stable settings hash, and per-area cost/walkability records. It can be constructed from a simple quad or from triangle input with per-triangle area ids, which lets the runtime bake collector preserve `NavMeshModifier` area overrides in the resulting polygons. It also exposes `debug_triangles()` so editor overlays can draw NavMesh area/tile triangles without understanding the serialized polygon layout, and `to_bytes()` / `from_bytes()` so `.znavmesh` artifacts round-trip through a binary payload instead of pretty JSON.

`NavigationGizmoSnapshot` projects baked navmesh triangles and off-mesh links into neutral debug geometry. The snapshot can convert itself into the existing `SceneGizmoOverlayExtract` line/pick-shape format using `SceneGizmoKind::NavigationMesh`. This establishes the DTO bridge from `.znavmesh` data to the viewport overlay surface; the renderer still decides which overlay records it draws.

`NavigationSettingsAsset` stores agent and area settings and is routed as a navigation settings resource. The runtime navigation plugin validates ids and finite numeric settings before installation. Bake output copies the active area costs into the navmesh asset so query code can apply the same walkability and cost semantics after the settings asset is no longer in memory.

## Design and Rationale

The runtime framework deliberately stays backend-neutral. Recast/Detour concepts appear as general DTOs, not as C++ handles or plugin-owned memory. This lets the runtime asset manager, editor UI, scripting layer, and plugin loader share the same language without forcing `zircon_runtime` to link a native navigation library.

Dynamic components remain JSON-backed. Vector, entity, and resource values now round-trip through JSON for plugin-authored components: arrays map to `Vec2`/`Vec3`/`Vec4`, `{ "entity": id }` maps to entity references, and `{ "resource": "..." }` maps to resource references.

## Control Flow

Editor or importer code produces `NavMeshAsset` and `NavigationSettingsAsset` records. The artifact store routes navmeshes into `navigation/navmeshes/*.znavmesh` using `NavMeshAsset` binary serialization and settings into `navigation/settings/*.toml`. Runtime plugins load those assets through the resource system and pass them to an implementation of `NavigationManager`.

Scene-facing tools write the five navigation component ids as dynamic components. Property editing uses the component descriptors registered by the navigation runtime plugin and the JSON conversion helpers in the world layer.

## Edge Cases

The framework does not bake geometry by itself and does not expose a compatibility straight-line placeholder API. Empty navmesh data is represented as a valid asset but runtime queries should return structured no-path results. Area masks are `u64`, so custom areas must remain below index `64`, and area cost records are serialized with the navmesh to keep query behavior independent of later settings mutations. Navigation gizmo conversion currently emits wire/pick data, not filled translucent triangle draw commands.

## Test Coverage

`cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never` passed with existing graphics/UI warnings. Inline module tests verify the default humanoid contract, fixed component id prefixing, and navmesh-to-overlay gizmo edge projection. Plugin native/runtime/editor checks are tracked in the plugin docs because they depend on the plugin workspace.
