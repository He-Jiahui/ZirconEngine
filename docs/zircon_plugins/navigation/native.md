---
related_code:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/tests.rs
  - zircon_plugins/navigation/native/tests/detour_query.rs
  - zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
implementation_files:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/detour.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/src/tests.rs
  - zircon_plugins/navigation/native/tests/detour_query.rs
  - zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER ... zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp ... -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke"
doc_type: module-detail
---

# Navigation Native Backend

## Purpose

`zircon_plugin_navigation_recast` is the native backend boundary for the navigation plugin. It vendors upstream Recast Navigation C++ sources and keeps the public Rust API expressed in Zircon DTOs (`NavMeshAsset`, `NavPathQuery`, `NavPathResult`, bake input records, and structured navigation errors).

## Native Boundary

`build.rs` compiles the vendored Recast, Detour, DetourCrowd, and DetourTileCache source folders plus `native/recast_bridge.cpp`, `native/recast_bake.cpp`, `native/detour_query.cpp`, and `native/detour_tile_cache.cpp` through the `cc` crate. It enables `DT_VIRTUAL_QUERYFILTER` so Zircon can keep 64-bit area masks and area-cost/walkability tables in custom Detour query filters instead of squeezing the public contract into Detour's default 16-bit polygon flags. The C ABI currently exposes:

- bridge version reporting
- a smoke check that allocates/frees Detour navmesh, DetourCrowd, and DetourTileCache objects and calls a Recast bounds helper
- native polyline length calculation used by the Rust facade path result
- native triangle-mesh baking that builds a Recast heightfield, compact heightfield, distance field, regions, contours, and polygon mesh, then returns Zircon-friendly flat vertex/index/polygon/tile buffers
- native Detour query ownership: `dtCreateNavMeshData`, `dtNavMesh`, and `dtNavMeshQuery` are created and freed behind an opaque C handle; path, sample-position, and raycast queries run through that handle and return copied Zircon-friendly result buffers
- native DetourTileCache obstacle carving: a copied single-tile compressed layer is built from Zircon navmesh polygons, box/cylinder obstacle requests are applied to a private `dtTileCache`, and the resulting mutable `dtNavMesh` is queried through an opaque C handle

`src/ffi.rs` owns the Rust ABI declarations and C layout records, `src/bake.rs` owns Rust-side bake input validation plus native-output conversion into `NavMeshAsset`, `src/detour.rs` owns the Rust RAII wrapper around the opaque Detour query handle, and `src/tile_cache.rs` owns the Rust RAII wrapper for the opaque TileCache query handle plus plugin-local obstacle DTOs. The upstream license is kept in `vendor/recastnavigation/License.txt`.

## Runtime Facade

The Rust facade still performs Zircon asset packaging and keeps deterministic graph queries as fallback support, but representable `NavMeshAsset` values now build an internal Detour tile/query object for pathfinding, nearest-position sampling, and walkability raycasts. It can bake simple fallback surfaces, rasterize collected triangle mesh input through native Recast into `.znavmesh` asset data with per-polygon areas, create a Detour corridor from copied asset buffers, apply 64-bit area masks and area costs through the custom query filter, reject mismatched agent-type queries, sample the nearest allowed Detour polygon inside query extents, raycast through Detour's surface query after preserving the facade's start-outside behavior, and include Detour off-mesh flags in path results.

`NavMeshAsset` carries copied area cost records from the active navigation settings. The backend uses those records to reject non-walkable areas and to weight polygon/link traversal, with link `cost_override` taking precedence when present. Runtime obstacle carving calls `RecastBackend::find_path_with_obstacles(...)`; when obstacles are present and the asset is representable by the TileCache bridge, `src/tile_cache.rs` builds the mutable native query and returns the carved Detour result before the normal non-carved Detour/Rust fallback path. Binary `NavMeshAsset::to_bytes()` / `from_bytes()` round-trip tests protect deterministic `.znavmesh` artifact payloads shared with the runtime asset store.

The native bake boundary normalizes downward-wound triangles before slope filtering so runtime-collected quads and imported mesh data do not disappear solely because of winding. The Detour query boundary also normalizes polygon winding before creating tile data, reconstructs shared-edge neighbours from Zircon polygon buffers, quantizes copied vertices into Detour single-tile data, and falls back to the Rust graph path when the asset shape is not representable by this first Detour bridge or when an off-mesh link uses a per-link `cost_override` that Detour's current filter path does not model exactly. The TileCache bridge is still transient: it reconstructs a single compressed tile layer from the loaded asset for each carved query instead of persisting tiled cache payloads inside `NavMeshAsset`. DetourCrowd simulation remains a follow-up native milestone behind the same facade.

## Validation

Prior to the TileCache slice, `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` passed: 16 unit tests, 3 Detour integration tests, and doctests. The tests cover native bridge linkage, simple-surface pathing, native Recast raster bake filtering for steep faces, non-finite bake source rejection before FFI, unique-vertex polygon adjacency for triangulated fan output, Detour string-pulled paths without Rust graph centroid waypoints, Detour sample projection, Detour raycast boundary hits, area masks, disconnected islands, off-mesh link bridging, agent mismatch errors, deterministic binary roundtrip, nearest-polygon sampling, vertical projection, triangle-edge projection, and raycast behavior that ignores off-mesh links while reporting straight-line gaps as hits.

For the TileCache slice, a focused Windows Cargo regression was attempted with `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_recast tile_cache_carved_obstacle_blocks_corridor_path --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never -- --exact --nocapture`, but it did not reach navigation test execution because the shared `zircon_runtime` dependency currently fails in unrelated renderer code (`scene_renderer_render_with_pipeline`). WSL has `g++ 11.4.0`; a direct native harness command compiling `zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp`, `native/detour_tile_cache.cpp`, `native/detour_query.cpp`, and vendored Recast/Detour sources with `-std=c++17 -DDT_VIRTUAL_QUERYFILTER` passed and printed `create status=1 polygons=3 obstacles=1` followed by `path status=2 ... TileCache path query found no complete path`. That harness protects the TileCache C ABI behavior until the unrelated renderer compile blocker allows Cargo to run the Rust regression.
