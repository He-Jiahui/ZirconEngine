---
related_code:
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_runtime/src/core/framework/navigation/bake.rs
  - zircon_runtime/src/core/framework/navigation/constants.rs
  - zircon_runtime/src/core/framework/navigation/error.rs
  - zircon_runtime/src/core/framework/navigation/gizmo.rs
  - zircon_runtime/src/core/framework/navigation/handle.rs
  - zircon_runtime/src/core/framework/navigation/manager.rs
  - zircon_runtime/src/core/framework/navigation/modifier.rs
  - zircon_runtime/src/core/framework/navigation/obstacle.rs
  - zircon_runtime/src/core/framework/navigation/off_mesh_link.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
  - zircon_runtime/src/core/framework/navigation/settings.rs
  - zircon_runtime/src/core/framework/navigation/stats.rs
  - zircon_runtime/src/core/framework/navigation/surface.rs
  - zircon_runtime/src/core/framework/navigation/tests.rs
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
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_runtime/src/core/framework/navigation/bake.rs
  - zircon_runtime/src/core/framework/navigation/constants.rs
  - zircon_runtime/src/core/framework/navigation/error.rs
  - zircon_runtime/src/core/framework/navigation/gizmo.rs
  - zircon_runtime/src/core/framework/navigation/handle.rs
  - zircon_runtime/src/core/framework/navigation/manager.rs
  - zircon_runtime/src/core/framework/navigation/modifier.rs
  - zircon_runtime/src/core/framework/navigation/obstacle.rs
  - zircon_runtime/src/core/framework/navigation/off_mesh_link.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
  - zircon_runtime/src/core/framework/navigation/settings.rs
  - zircon_runtime/src/core/framework/navigation/stats.rs
  - zircon_runtime/src/core/framework/navigation/surface.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
tests:
  - zircon_runtime/src/core/framework/navigation/tests.rs
  - off_mesh_bridge_descriptor_is_a_first_class_navigation_contract
  - automatic_agent_tick_does_not_cross_manual_off_mesh_links
  - automatic_agent_tick_respects_auto_traverse_links_opt_out
  - explicit_path_query_can_still_cross_manual_off_mesh_links
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/navigation/*.rs (2026-06-04 navigation boundary split: passed)
  - git diff --check -- zircon_runtime/src/core/framework/navigation docs/zircon_runtime/core/framework/navigation.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 navigation boundary split: passed with expected LF-to-CRLF warnings)
  - cargo test -p zircon_runtime --lib navigation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-navigation-framework-split --message-format short --color never (planned for current navigation boundary split)
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

The navigation framework now lives in a folder-backed subtree. `mod.rs` is only the structural boundary and public re-export surface. The child files own narrow responsibilities:

- `constants.rs` and `handle.rs` define stable ids, area masks, and navmesh handles.
- `settings.rs`, `surface.rs`, `modifier.rs`, `agent.rs`, `obstacle.rs`, and `off_mesh_link.rs` define the authoring and runtime component DTO families.
- `bake.rs`, `query.rs`, `stats.rs`, `error.rs`, and `manager.rs` define runtime operations and their neutral result/error records.
- `gizmo.rs` converts baked navmesh debug data into the shared scene gizmo overlay contract.
- `tests.rs` keeps the framework-level contract checks out of the root wiring file.

Baked data lives in `zircon_runtime/src/asset/assets/navigation.rs` and is exposed through `ImportedAsset::{NavMesh, NavigationSettings}` plus `ResourceKind::{NavMesh, NavigationSettings}`. Dynamic component property JSON conversion is extended in `zircon_runtime/src/scene/world/dynamic_components.rs`.

## Behavior Model

The framework defines six fixed dynamic component type ids:

- `navigation.Component.NavMeshSurface`
- `navigation.Component.NavMeshModifier`
- `navigation.Component.NavMeshAgent`
- `navigation.Component.NavMeshObstacle`
- `navigation.Component.NavMeshOffMeshLink`
- `navigation.Component.NavMeshOffMeshBridge`

The default humanoid agent matches the plan values: radius `0.5`, height `2.0`, climb `0.4`, slope `45`, speed `3.5`, acceleration `8.0`, angular speed `360`, and stopping distance `0.1`. Areas reserve `0` for `not_walkable`, `1` for `walkable`, `2` for `jump`, and `3..63` for custom areas.

Off-mesh links model a single traversal edge. Off-mesh bridges are a related authoring contract for wider multi-lane crossings: they keep the same endpoint, area, cost, bidirectionality, activation, agent-type, and traversal-mode semantics, then add `lane_count` so the runtime plugin can expand one bridge descriptor into bounded per-lane baked links. `MAX_OFF_MESH_BRIDGE_LANES` caps expansion at `32` lanes, keeping editor-authored bridge values from producing unbounded bake artifacts. `NavLinkTraversalMode` and `NavMeshAgentDescriptor::auto_traverse_links` are neutral policy inputs only: the framework preserves them in components and baked assets, while the active navigation runtime decides whether automatic movement may consume those links.

`NavMeshAsset` stores deterministic baked data: vertices, indices, polygons, tiles, off-mesh links, agent type, a stable settings hash, and per-area cost/walkability records. It can be constructed from a simple quad or from triangle input with per-triangle area ids, which lets the runtime bake collector preserve `NavMeshModifier` area overrides in the resulting polygons. It also exposes `debug_triangles()` so editor overlays can draw NavMesh area/tile triangles without understanding the serialized polygon layout, and `to_bytes()` / `from_bytes()` so `.znavmesh` artifacts round-trip through a binary payload instead of pretty JSON.

`NavigationGizmoSnapshot` projects baked navmesh triangles and off-mesh links into neutral debug geometry. The snapshot can convert itself into the existing `SceneGizmoOverlayExtract` line/pick-shape format using `SceneGizmoKind::NavigationMesh`. This establishes the DTO bridge from `.znavmesh` data to the viewport overlay surface; the renderer still decides which overlay records it draws.

`NavigationSettingsAsset` stores agent and area settings and is routed as a navigation settings resource. The runtime navigation plugin validates ids and finite numeric settings before installation. Bake output copies the active area costs into the navmesh asset so query code can apply the same walkability and cost semantics after the settings asset is no longer in memory.

## Design and Rationale

The runtime framework deliberately stays backend-neutral. Recast/Detour concepts appear as general DTOs, not as C++ handles or plugin-owned memory. This lets the runtime asset manager, editor UI, scripting layer, and plugin loader share the same language without forcing `zircon_runtime` to link a native navigation library.

`NavMeshAgentDescriptor` is intentionally limited to authoring and configuration fields such as speed, acceleration, angular speed, stopping distance, avoidance flags, link traversal preference, and destination. Concrete per-entity velocity, acceleration integration, arrival braking, rotation interpolation, and automatic off-mesh traversal filtering are owned by the active navigation runtime plugin. This keeps serialized dynamic components and framework DTOs stable while allowing DetourCrowd-style, custom ECS steering, or gameplay-scripted manual-link backends to maintain their own simulation state.

The folder layout follows three reference-engine signals. Godot separates navigation agents, links, obstacles, and regions as distinct scene components; Unreal separates `NavigationSystem`, `NavMesh`, and `NavLink` families; Fyrox keeps navigational mesh runtime data as a dedicated scene subsystem instead of merging it into generic scene nodes. Zircon keeps the same domain split but lands it in the runtime framework contract layer so plugins and editor tooling share stable Rust DTOs.

Dynamic components remain JSON-backed. Vector, entity, and resource values now round-trip through JSON for plugin-authored components: arrays map to `Vec2`/`Vec3`/`Vec4`, `{ "entity": id }` maps to entity references, and `{ "resource": "..." }` maps to resource references.

## Control Flow

Editor or importer code produces `NavMeshAsset` and `NavigationSettingsAsset` records. The artifact store routes navmeshes into `navigation/navmeshes/*.znavmesh` using `NavMeshAsset` binary serialization and settings into `navigation/settings/*.toml`. Runtime plugins load those assets through the resource system and pass them to an implementation of `NavigationManager`.

Scene-facing tools write the six navigation component ids as dynamic components. Property editing uses the component descriptors registered by the navigation runtime plugin and the JSON conversion helpers in the world layer.

## Edge Cases

The framework does not bake geometry by itself and does not expose a compatibility straight-line placeholder API. Empty navmesh data is represented as a valid asset but runtime queries should return structured no-path results. Area masks are `u64`, so custom areas must remain below index `64`, and area cost records are serialized with the navmesh to keep query behavior independent of later settings mutations. Navigation gizmo conversion currently emits wire/pick data, not filled translucent triangle draw commands.

## Test Coverage

Historical navigation validation: `cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never` passed with existing graphics/UI warnings. Inline framework tests verify the default humanoid contract, fixed component id prefixing, off-mesh bridge default/serde semantics, and navmesh-to-overlay gizmo edge projection. Plugin native/runtime/editor checks are tracked in the plugin docs because they depend on the plugin workspace.

Current boundary split static validation passed: scoped rustfmt over `zircon_runtime/src/core/framework/navigation/*.rs`, a conflict-marker scan, and `git diff --check` over the touched navigation/doc/session files. The focused `cargo test -p zircon_runtime --lib navigation` run is still pending until active Cargo lanes from other sessions have enough capacity.

2026-06-04 plugin runtime follow-up split `zircon_plugins/navigation/runtime/src/manager/bake.rs` into a structural bake facade plus `manager/bake/{asset,diagnostics,filter,geometry,modifier,surface}.rs`. This did not change the framework DTOs; it keeps plugin-owned scene scans, Recast/simple fallback dispatch, off-mesh embedding, and bake diagnostics out of the neutral `zircon_runtime::core::framework::navigation` contract layer. Focused plugin Cargo validation is still pending while active Cargo lanes from other sessions are running.

2026-06-07 plugin runtime follow-up added manager-private agent motion state and focused acceleration/auto-braking coverage in `zircon_plugins/navigation/runtime/src/tests/manager.rs`. The framework contract did not change; this document records the boundary that runtime velocity is plugin-owned state, not a new serialized `NavMeshAgentDescriptor` field.

2026-06-07 plugin runtime traversal follow-up added automatic off-mesh traversal filtering in `zircon_plugins/navigation/runtime/src/manager/traversal.rs`. The framework contract still did not change: manual links remain serialized asset/component data and explicit path queries can still plan across them, while automatic agent ticks are the plugin-owned policy surface that filters manual links and `auto_traverse_links = false` paths.
