---
related_code:
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/Cargo.toml
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/asset/assets/navigation.rs
implementation_files:
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --message-format short --color never
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --target-dir E:\cargo-targets\zircon-navigation-native-check --jobs 1 --message-format short --color never
  - cargo check -p zircon_runtime --message-format short --color never
doc_type: module-detail
---

# Navigation Runtime Plugin

## Purpose

The navigation runtime plugin owns the first real navigation manager surface for Zircon. It registers the `NavigationModule`, contributes five dynamic component descriptors, loads baked navmesh assets, exposes path/sample/raycast APIs through the shared `NavigationManager` trait, and ticks basic scene agents toward destinations.

## Related Files

`src/lib.rs` wires the plugin descriptor, module descriptor, package manifest helpers, and extension registration. `src/components.rs` declares the editable properties for `NavMeshSurface`, `NavMeshModifier`, `NavMeshAgent`, `NavMeshObstacle`, and `NavMeshOffMeshLink`. `src/manager.rs` implements `DefaultNavigationManager`.

`navigation/native` is the Recast backend boundary. It vendors upstream Recast Navigation C++ sources for Recast, Detour, DetourCrowd, and DetourTileCache under `vendor/recastnavigation`, compiles them through `cc`, and exposes a small C ABI bridge in `native/recast_bridge.cpp`. The Rust facade still owns Zircon asset conversion and deterministic query tests, but it now verifies that the native Recast/Detour modules are linked and reachable.

## Behavior Model

Registration contributes:

- `NavigationModule` with lazy manager `NavigationModule.Manager.NavigationManager`
- runtime capabilities `runtime.plugin.navigation` and `runtime.plugin.navigation.recast`
- the five `navigation.Component.*` dynamic component descriptors
- plugin options for the default agent type, default settings asset, debug gizmos, and bake backend
- event catalog entries for bake completion, path query completion/failure, and agent ticks

`DefaultNavigationManager` keeps loaded `NavMeshAsset` values in a mutex-protected map and returns stable `NavMeshHandle` values. Queries can address a specific handle or fall back to the first loaded mesh. Empty maps return a structured missing-navmesh error; empty assets or blocked area masks return `NoPath`.

The current backend supports deterministic simple-surface fallback baking, triangle-mesh bake packaging, polygon-graph path queries, area-mask filtering, disconnected-island no-path results, and off-mesh links that bridge otherwise disconnected polygons. Agent ticking reads `NavMeshAgent` dynamic components, follows an optional `destination`, respects `speed`, `stopping_distance`, and `update_position`, and updates the entity transform through `World::update_transform`.

## Design and Rationale

The plugin keeps navigation behavior out of `zircon_runtime` while still making the manager visible through the existing module/service system. This follows the independent-plugin direction and lets editor, scripting, and future native backends use one neutral API.

The native C++ boundary is intentionally narrow. It proves that upstream Recast/Detour/DetourCrowd/DetourTileCache can be built as part of the plugin and called from Rust without leaking native handles into shared DTOs. Full Recast rasterization, Detour navmesh query objects, TileCache obstacle mutation, and DetourCrowd simulation remain backend-internal upgrades behind this same facade.

## Control Flow

At plugin registration time, `NavigationRuntimePlugin::register_runtime_extensions` registers the module and every component descriptor. When the manager bakes a surface, it scans scene dynamic components for enabled `NavMeshSurface` descriptors, chooses the requested surface or the first enabled surface, and collects bake geometry according to the surface collection mode. Render-mesh mode uses cube/mesh node footprints when model vertex payloads are not available through the world; collider mode uses box/sphere/capsule collider footprints. The collector excludes navigation agents, obstacles, and off-mesh-link nodes from source geometry; applies nearest `NavMeshModifier` remove/area override rules; embeds active off-mesh links into the asset; then returns a `NavMeshBakeReport` with source counts and diagnostics.

At runtime query time, `load_nav_mesh` stores the asset, `find_path` delegates to the backend, `sample_position` clamps to baked bounds, and `raycast` reports clear or blocked traversal. The backend builds a lightweight polygon graph from shared polygon edges and declared off-mesh links, then reconstructs a waypoint list that flags off-mesh traversal points. Agent ticking is intentionally conservative and mutates only dynamic entities whose navigation component declares a destination. Component parsing normalizes editor property tags such as `{ "resource": "..." }` and `{ "entity": 12 }` before deserializing navigation descriptors.

## Edge Cases

Agent movement can be blocked by missing transforms or immutable/static entity transforms, and those failures are reported in `NavAgentTickReport`. The bake collector can preserve scene component intent, but it is not yet a true Recast voxel/raster pipeline over imported render mesh vertices. The manager does not yet implement tiled Detour queries, tile-cache obstacle carving, DetourCrowd avoidance, or native off-mesh link traversal; the DTO and component surface are in place for those follow-up backend upgrades.

## Test Coverage

`cargo check -p zircon_runtime --message-format short --color never` passed earlier in this navigation slice.

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --message-format short --color never` progressed into the navigation runtime build. The first run exposed unrelated active workspace issues in asset meta/plugin workspace scaffolding; after narrow fixes it reached the navigation crate and exposed a private `NodeKind` test import, which has been corrected. A final plugin test rerun was attempted with a separate target directory, but concurrent Cargo jobs in other sessions caused timeout/lock pressure before completion.

`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --target-dir E:\cargo-targets\zircon-navigation-native-check --jobs 1 --message-format short --color never` was attempted after vendoring the C++ bridge; it timed out after 10 minutes while the machine had multiple unrelated workspace Cargo/rustc jobs active, so native compile success is not yet claimed.
