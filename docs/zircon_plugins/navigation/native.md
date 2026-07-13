---
related_code:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/asset_ffi.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/crowd.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/detour_result.rs
  - zircon_plugins/navigation/native/src/fallback_query.rs
  - zircon_plugins/navigation/native/src/fallback_query/geometry.rs
  - zircon_plugins/navigation/native/src/fallback_query/graph.rs
  - zircon_plugins/navigation/native/src/fallback_query/path.rs
  - zircon_plugins/navigation/native/src/fallback_query/raycast.rs
  - zircon_plugins/navigation/native/src/fallback_query/sampling.rs
  - zircon_plugins/navigation/native/src/fallback_query/validation.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/tests/mod.rs
  - zircon_plugins/navigation/native/src/tests/asset.rs
  - zircon_plugins/navigation/native/src/tests/bake.rs
  - zircon_plugins/navigation/native/src/tests/linkage.rs
  - zircon_plugins/navigation/native/src/tests/path.rs
  - zircon_plugins/navigation/native/src/tests/raycast.rs
  - zircon_plugins/navigation/native/src/tests/sampling.rs
  - zircon_plugins/navigation/native/src/tests/support.rs
  - zircon_plugins/navigation/native/src/tests/tile_cache.rs
  - zircon_plugins/navigation/native/tests/detour_query.rs
  - zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.cpp
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.h
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache_raster.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache_raster.h
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_runtime/src/core/framework/navigation/asset/mod.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
implementation_files:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/asset_ffi.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/crowd.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/detour_result.rs
  - zircon_plugins/navigation/native/src/fallback_query.rs
  - zircon_plugins/navigation/native/src/fallback_query/geometry.rs
  - zircon_plugins/navigation/native/src/fallback_query/graph.rs
  - zircon_plugins/navigation/native/src/fallback_query/path.rs
  - zircon_plugins/navigation/native/src/fallback_query/raycast.rs
  - zircon_plugins/navigation/native/src/fallback_query/sampling.rs
  - zircon_plugins/navigation/native/src/fallback_query/validation.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/tests/mod.rs
  - zircon_plugins/navigation/native/src/tests/asset.rs
  - zircon_plugins/navigation/native/src/tests/bake.rs
  - zircon_plugins/navigation/native/src/tests/linkage.rs
  - zircon_plugins/navigation/native/src/tests/path.rs
  - zircon_plugins/navigation/native/src/tests/raycast.rs
  - zircon_plugins/navigation/native/src/tests/sampling.rs
  - zircon_plugins/navigation/native/src/tests/support.rs
  - zircon_plugins/navigation/native/tests/detour_query.rs
  - zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.cpp
  - zircon_plugins/navigation/native/native/detour_off_mesh_connections.h
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache_raster.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache_raster.h
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - zircon_plugins/navigation/native/src/tests/mod.rs
  - zircon_plugins/navigation/native/src/tests/asset.rs
  - zircon_plugins/navigation/native/src/tests/bake.rs
  - zircon_plugins/navigation/native/src/tests/linkage.rs
  - zircon_plugins/navigation/native/src/tests/path.rs
  - zircon_plugins/navigation/native/src/tests/raycast.rs
  - zircon_plugins/navigation/native/src/tests/sampling.rs
  - zircon_plugins/navigation/native/src/tests/support.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER ... zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp ... -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke"
doc_type: module-detail
---

# Navigation Native Backend

## Purpose

`zircon_plugin_navigation_recast` is the native backend boundary for the navigation plugin. It vendors upstream Recast Navigation C++ sources and keeps the public Rust API expressed in Zircon DTOs (`NavMeshAsset`, `NavPathQuery`, `NavPathResult`, bake input records, and structured navigation errors).

## Native Boundary

`build.rs` compiles the vendored Recast, Detour, DetourCrowd, and DetourTileCache source folders plus `native/recast_bridge.cpp`, `native/recast_bake.cpp`, `native/detour_query.cpp`, `native/detour_off_mesh_connections.cpp`, `native/detour_crowd.cpp`, `native/detour_tile_cache.cpp`, and the isolated `native/detour_tile_cache_raster.cpp` layer builder through the `cc` crate. It enables `DT_VIRTUAL_QUERYFILTER`. Zircon's documented vendored Detour extension adds a 64-bit area mask to the value-stored `dtQueryFilter`, allowing DetourCrowd's 16 filter slots to preserve per-agent masks while each slot receives the asset's area-cost and walkability table. The C ABI currently exposes:

- bridge version reporting
- a smoke check that allocates/frees Detour navmesh, DetourCrowd, and DetourTileCache objects and calls a Recast bounds helper
- native polyline length calculation used by the Rust facade path result
- native triangle-mesh baking that builds a Recast heightfield, compact heightfield, distance field, regions, contours, and polygon mesh, then returns Zircon-friendly flat vertex/index/polygon/tile buffers
- native Detour query ownership: `dtCreateNavMeshData`, `dtNavMesh`, and `dtNavMeshQuery` are created and freed behind an opaque C handle; path, sample-position, and raycast queries run through that handle and return copied Zircon-friendly result buffers
- native DetourCrowd ownership: `zr_nav_crowd_create` transfers one Detour query/navmesh owner into an opaque Crowd handle; add/remove/target operations remain explicit, while each frame performs one Crowd update and one batch read of position, path desired velocity, avoidance velocity, and acceleration-limited velocity
- native DetourTileCache obstacle carving: a copied single-tile compressed layer is built from Zircon navmesh polygons, box/cylinder obstacle requests are applied to a private `dtTileCache`, and the resulting mutable `dtNavMesh` is queried through an opaque C handle
- shared off-mesh connection packing: stable non-zero asset link ids become Detour user ids, ordinary query tiles and TileCache rebuilds bind through the same owner, and straight-path results return the concrete link id rather than only a generic flag
- direct ABI pointer/count validation rejects a non-zero off-mesh count with no link buffer before bounds scanning

`src/lib.rs` is the public facade for bridge version checks, `RecastBackend`, `RecastCrowd`, bake DTO exports, and TileCache obstacle DTO exports. `src/ffi.rs` owns the Rust ABI declarations and C layout records, `src/bake.rs` owns Rust-side bake input validation plus native-output conversion into `NavMeshAsset`, `src/asset_ffi.rs` owns the shared asset-to-Detour input packing, `src/detour_result.rs` owns native Detour path-result conversion, `src/detour.rs` owns the Rust RAII wrapper around the opaque Detour query handle, `src/crowd.rs` owns the unique mutable Crowd RAII wrapper and batch state conversion, and `src/tile_cache.rs` owns the persistent mutable `RecastTileCache`, stable obstacle handles, explicit add/remove/update operations, and carved path queries. `src/fallback_query.rs` remains the isolated deterministic fallback. The upstream license is kept in `vendor/recastnavigation/License.txt`.

## Runtime Facade

The Rust facade still performs Zircon asset packaging and keeps deterministic graph queries as fallback support, but representable `NavMeshAsset` values now build an internal Detour tile/query object for pathfinding, nearest-position sampling, and walkability raycasts. It can bake simple fallback surfaces, rasterize collected triangle mesh input through native Recast into `.znavmesh` asset data with per-polygon areas, create a Detour corridor from copied asset buffers, apply 64-bit area masks and area costs through the custom query filter, reject mismatched agent-type queries, sample the nearest allowed Detour polygon inside query extents, raycast through Detour's surface query after preserving the facade's start-outside behavior, and include Detour off-mesh flags in path results. When the native Detour wrapper cannot represent an asset exactly, the fallback path stays isolated under `src/fallback_query/` instead of growing the public crate root.

`NavMeshAsset` carries copied area cost records from the active navigation settings. `NavQueryFilter` adds caller-owned 64-entry cost overrides and Detour-compatible include/exclude flags; native Detour and Rust fallback routing apply the same filter while asset walkability remains authoritative and link `cost_override` takes precedence. Ordinary queries continue to seed their filter from the baked asset cost table. Fallback shared edges are directed and charge their source polygon area, matching Detour in both query directions. Runtime obstacle carving keeps one mutable TileCache owner per loaded navmesh, synchronizes entity-keyed obstacle handles, rebuilds after add/remove/change, and preserves the owner so removal restores the corridor. Obstacle handles carry a private cache identity and cannot be replayed against another owner. The cache reserves 128 runtime obstacle slots; the safe wrapper and native batch creator flush Detour's fixed 64-request queue before overflow, and runtime replacement flushes removals before allocating new refs. Binary `NavMeshAsset::to_bytes()` / `from_bytes()` round-trip tests protect deterministic `.znavmesh` artifact payloads shared with the runtime asset store.

The native bake boundary normalizes downward-wound triangles before slope filtering so runtime-collected quads and imported mesh data do not disappear solely because of winding. The Detour query boundary also normalizes polygon winding before creating tile data, reconstructs shared-edge neighbours from Zircon polygon buffers, quantizes copied vertices into Detour single-tile data, and falls back to the Rust graph path when the asset shape is not representable. The TileCache bridge is a persistent, uniquely owned mutable world; raster/layer construction is separated from its lifecycle, mutation, and query orchestration. Crowd bridge ABI v2 follows explicit ownership rules: ownership of the Detour query transfers only after successful Crowd creation, the Crowd retains that owner for its full lifetime, targets use the agent's assigned area filter, and every throwing command boundary converts native exceptions into a copied error record. Controller-owned positions use `dtPathCorridor::movePosition` through `sync_agent_position`, preserving the current corridor while feeding back the authoritative transform. The Rust API is `Send` but not `Sync` because every native mutation requires unique mutable ownership.

## Validation

Prior to the TileCache slice, `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` passed: 16 unit tests, 3 Detour integration tests, and doctests. The current unit tests are split under `src/tests/` by behavior area. They cover native bridge linkage, simple-surface pathing, native Recast raster bake filtering for steep faces, non-finite bake source rejection before FFI, unique-vertex polygon adjacency for triangulated fan output, Detour string-pulled paths without Rust graph centroid waypoints, Detour sample projection, Detour raycast boundary hits, area masks, disconnected islands, off-mesh link bridging, agent mismatch errors, deterministic binary roundtrip, nearest-polygon sampling, vertical projection, triangle-edge projection, and raycast behavior that ignores off-mesh links while reporting straight-line gaps as hits.

The fresh M3 Windows-native validator job `2dc36f6c89f44ffe837b19fc69284c60` passed `cargo test -p zircon_plugin_navigation_recast --locked` with 22 unit and 4 integration tests. It covers Crowd movement/state batch round-trip, rejection when an agent mask excludes the surface, controller-position corridor synchronization, filter-slot recycling, bridge ABI v2, Detour queries, TileCache behavior, and doctests.

The final M4 Windows-native validator job `5c1a96ab19e54cb1bb47d091979e17d7` passed `cargo test -p zircon_plugin_navigation_recast --locked`: 31 unit tests, 4 integration tests, and doctests. Exact M4/review anchors cover obstacle add/carve, remove/restore, 65-request incremental and batch queue boundaries, cache-scoped handles, baked-cost preservation, native and fallback bidirectional area-cost routing, and include/exclude flag filtering.

M5 raises the internal native bridge version to 4. `detour_off_mesh_connections` is the single owner of endpoint validation, radius/area/direction arrays, stable user ids, and per-tile start-point filtering. `detour_query.cpp` no longer grows another protocol responsibility, validates pointer/count pairs before bounds iteration, and TileCache no longer rejects every asset that contains links. `offmesh_link_present_in_baked_tiles` and the malformed direct-ABI regression verify the mutable tile path and FFI guard; the M5 testing-stage result is recorded in the owning child-plan output record.

For the 2026-06-04 native facade/fallback split, static validation passed with `rustfmt --edition 2021 --check` over `src/lib.rs`, `src/asset_ffi.rs`, `src/detour_result.rs`, `src/fallback_query.rs`, every file under `src/fallback_query/`, `src/detour.rs`, and `src/tile_cache.rs`; `git diff --check` passed for the touched native Rust files, this doc, and the active session note with only expected line-ending warnings. A low-concurrency `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-native-facade-fallback-split-0604 --message-format short --color never` attempt timed out after ten minutes before returning Rust diagnostics. A process audit immediately afterward showed active Cargo/rustc lanes belonged to other target directories, so compile/test acceptance for this relocation is still pending.
