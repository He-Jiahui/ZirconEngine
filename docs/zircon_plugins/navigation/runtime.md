---
related_code:
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/component_json.rs
  - zircon_plugins/navigation/runtime/src/runtime_obstacles.rs
  - zircon_plugins/navigation/runtime/src/settings_hash.rs
  - zircon_plugins/navigation/runtime/src/settings_validation.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/tests/detour_query.rs
  - zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/Cargo.toml
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/asset/artifact/store.rs
implementation_files:
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/component_json.rs
  - zircon_plugins/navigation/runtime/src/runtime_obstacles.rs
  - zircon_plugins/navigation/runtime/src/settings_hash.rs
  - zircon_plugins/navigation/runtime/src/settings_validation.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/asset/artifact/store.rs
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never
  - wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER ... zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp ... -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke"
doc_type: module-detail
---

# Navigation Runtime Plugin

## Purpose

The navigation runtime plugin owns the first real navigation manager surface for Zircon. It registers the `NavigationModule`, contributes five dynamic component descriptors, loads baked navmesh assets, exposes path/sample/raycast APIs through the shared `NavigationManager` trait, and ticks basic scene agents toward destinations.

## Related Files

`src/lib.rs` wires the plugin descriptor, module descriptor, package manifest helpers, and extension registration. `src/components.rs` declares the editable properties for `NavMeshSurface`, `NavMeshModifier`, `NavMeshAgent`, `NavMeshObstacle`, and `NavMeshOffMeshLink`. `src/manager.rs` implements `DefaultNavigationManager`; `src/component_json.rs` normalizes tagged editor property JSON before component deserialization; `src/runtime_obstacles.rs` collects runtime obstacle descriptors and converts carving obstacles into Recast-native obstacle records; `src/settings_hash.rs` and `src/settings_validation.rs` keep stable bake-hash and settings-validation logic out of the manager file.

`navigation/native` is the Recast/Detour backend boundary. It vendors upstream Recast Navigation C++ sources for Recast, Detour, DetourCrowd, and DetourTileCache under `vendor/recastnavigation`, compiles them through `cc`, and exposes C ABI bridge files under `native/`. `src/bake.rs` validates bake input and converts native Recast bake buffers into `NavMeshAsset`; `src/detour.rs` wraps an opaque native `dtNavMesh` / `dtNavMeshQuery` owner for path/sample/raycast queries; `src/tile_cache.rs` wraps an opaque native TileCache query owner for carved obstacle path queries; `src/ffi.rs` owns the ABI declarations. Triangle-mesh bakes pass through Recast heightfield, region, contour, and polygon mesh construction before returning Zircon DTOs, and representable navmesh assets now query through Detour or transient DetourTileCache before falling back to the deterministic Rust graph.

## Behavior Model

Registration contributes:

- `NavigationModule` with lazy manager `NavigationModule.Manager.NavigationManager`
- runtime capabilities `runtime.plugin.navigation` and `runtime.plugin.navigation.recast`
- the five `navigation.Component.*` dynamic component descriptors
- plugin options for the default agent type, default settings asset, debug gizmos, and bake backend
- event catalog entries for bake completion, path query completion/failure, and agent ticks

`DefaultNavigationManager` keeps loaded `NavMeshAsset` values in a mutex-protected map and returns stable `NavMeshHandle` values. Queries can address a specific handle or fall back deterministically to the lowest loaded handle. Empty maps return a structured missing-navmesh error; empty assets or blocked area masks return `NoPath`.

The current backend supports deterministic simple-surface fallback baking, native Recast raster/poly-mesh baking for collected triangle geometry, stable settings-hash stamping, area-cost copying, Detour-backed path/sample/raycast queries for representable assets, transient DetourTileCache obstacle carving for runtime path queries that include carving obstacles, Rust polygon-graph fallback queries for unsupported asset shapes or off-mesh cost-override cases, area-mask filtering, disconnected-island no-path results, optional off-mesh link embedding controlled by `NavMeshSurface.generate_links`, and off-mesh links that bridge otherwise disconnected polygons. Agent ticking reads `NavMeshAgent` dynamic components, follows an optional `destination`, prefers the next path waypoint from the loaded navmesh when available, falls back to direct movement only when no navmesh is loaded, blocks and reports agents when a loaded navmesh returns no path or an invalid query, respects `speed`, `stopping_distance`, `update_position`, and `update_rotation`, performs basic obstacle/agent separation, and updates the entity transform through `World::update_transform`.

## Design and Rationale

The plugin keeps navigation behavior out of `zircon_runtime` while still making the manager visible through the existing module/service system. This follows the independent-plugin direction and lets editor, scripting, and future native backends use one neutral API.

The native C++ boundary is intentionally narrow. It proves that upstream Recast/Detour/DetourCrowd/DetourTileCache can be built as part of the plugin and called from Rust without leaking native handles into shared DTOs. Recast rasterization backs triangle-mesh bakes, Detour navmesh query objects back normal runtime queries through an opaque native owner, and TileCache obstacle mutation now backs carved obstacle path queries through a separate opaque native owner. DetourCrowd simulation remains a backend-internal upgrade behind this same facade.

## Control Flow

At plugin registration time, `NavigationRuntimePlugin::register_runtime_extensions` registers the module and every component descriptor. `load_navigation_settings` validates unique agent/area ids, finite numeric settings, non-empty names, and maskable area ids before installing settings. When the manager bakes a surface, it scans scene dynamic components for enabled `NavMeshSurface` descriptors, chooses the requested surface or the first enabled surface, validates the requested agent type against `NavigationSettingsAsset`, and collects bake geometry according to the surface collection mode. Render-mesh mode uses cube/mesh node footprints when model vertex payloads are not available through the world; collider mode uses box/sphere/capsule collider footprints. The collector excludes navigation surface authoring volumes, agents, obstacles, and off-mesh-link nodes from source geometry; applies nearest `NavMeshModifier` remove/area override rules; treats a modifier on the selected surface as a bake-scope area override; removes static bake sources intersecting carving obstacles; then sends remaining triangles through native Recast rasterization. The native boundary normalizes downward-wound triangles before slope filtering, builds Recast regions/contours/polygons, and returns tile/polygon buffers. The manager optionally embeds active off-mesh links, stamps an explicit stable FNV-style hash over surface/settings fields, copies area costs into the asset, and returns a `NavMeshBakeReport` with source counts and diagnostics.

At runtime query time, `load_nav_mesh` stores the asset, `find_path` delegates to the backend, `sample_position` finds the nearest allowed polygon sample inside query extents, and `raycast` reports clear or blocked traversal. The backend first builds a native Detour query owner from copied asset buffers, reconstructs shared-edge neighbour data, applies Zircon area masks/costs through a virtual Detour filter, and uses `findPath` / `findStraightPath`, `findNearestPoly`, or `raycast` depending on the request. When the world contains active carving obstacles, `DefaultNavigationManager` collects them through `runtime_obstacles.rs` and calls `RecastBackend::find_path_with_obstacles(...)`; that path builds a transient TileCache layer from the loaded asset, applies box/capsule obstacles, updates the mutable navmesh, and queries the carved corridor before falling back to the normal non-carved path if the asset is not representable by the TileCache bridge. When the asset cannot be represented by the Detour bridge, the backend falls back to the lightweight Rust polygon graph and sampled visibility path, preserving off-mesh `cost_override` behavior and previous no-path semantics. Agent ticking is intentionally conservative and mutates only dynamic entities whose navigation component declares a destination. Runtime stats track loaded navmeshes, scanned active agents, active obstacles, and active off-mesh links. Component parsing normalizes editor property tags such as `{ "resource": "..." }` and `{ "entity": 12 }` before deserializing navigation descriptors.

## Edge Cases

Agent movement can be blocked by missing transforms or immutable/static entity transforms, and those failures are reported in `NavAgentTickReport`. Obstacle support is intentionally scoped: bake-time carving removes intersecting collected static sources before rasterization, runtime path queries can carve transient TileCache obstacles for loaded navmeshes, and runtime avoidance applies simple local separation from obstacle centers and neighboring agents. The bake backend is now a real Recast voxel/raster/poly-mesh pipeline for the triangles the runtime collector provides, and the query backend owns Detour/TileCache query objects internally, but render-mesh collection still falls back to cube/mesh node footprints until imported model vertex payloads are exposed through the world or asset pipeline. The manager does not yet persist tiled Detour cache data in `NavMeshAsset`, update obstacles incrementally across frames, run DetourCrowd simulation, or support full native off-mesh cost/corridor traversal; the DTO and component surface are in place for those follow-up backend upgrades.

## Test Coverage

Prior to the TileCache slice, `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` passed: 12 unit tests and doctests. The tests cover runtime registration, dynamic component descriptor JSON conversion, typed resource properties, native Recast-backed surface baking, bake modifier/off-mesh-link embedding, link-generation disablement and settings hash stamping, obstacle carving, basic obstacle avoidance/stats, path queries over loaded navmeshes, deterministic default mesh selection, loaded-navmesh no-path agent blocking, invalid settings rejection, and agent ticking.

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` also passed before the TileCache slice with 16 unit tests, 3 Detour integration tests, and doctests, including native Recast raster bake filtering for steep faces, non-finite source rejection before FFI, unique-vertex polygon adjacency for triangulated fan output, Detour string-pulled corridor paths, Detour sample projection, and Detour raycast boundary hits. `cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never` passed previously with existing graphics/UI warnings.

For the TileCache runtime slice, `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never -- --exact --nocapture` was attempted but did not reach navigation test execution because the shared `zircon_runtime` dependency currently fails in unrelated renderer code (`scene_renderer_render_with_pipeline`). A lower-layer WSL native harness compiling `zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp`, `native/detour_tile_cache.cpp`, `native/detour_query.cpp`, and vendored Recast/Detour sources with `g++ 11.4.0` did execute the same corridor/obstacle shape at the C ABI layer and returned `path status=2 ... TileCache path query found no complete path`. Runtime Cargo acceptance remains blocked until the unrelated renderer compile drift is resolved.
