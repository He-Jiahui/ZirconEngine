---
related_code:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
implementation_files:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/src/bake.rs
  - zircon_plugins/navigation/native/src/ffi.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/recast_bake.cpp
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
doc_type: module-detail
---

# Navigation Native Backend

## Purpose

`zircon_plugin_navigation_recast` is the native backend boundary for the navigation plugin. It vendors upstream Recast Navigation C++ sources and keeps the public Rust API expressed in Zircon DTOs (`NavMeshAsset`, `NavPathQuery`, `NavPathResult`, bake input records, and structured navigation errors).

## Native Boundary

`build.rs` compiles the vendored Recast, Detour, DetourCrowd, and DetourTileCache source folders plus `native/recast_bridge.cpp` and `native/recast_bake.cpp` through the `cc` crate. The C ABI currently exposes:

- bridge version reporting
- a smoke check that allocates/frees Detour navmesh, DetourCrowd, and DetourTileCache objects and calls a Recast bounds helper
- native polyline length calculation used by the Rust facade path result
- native triangle-mesh baking that builds a Recast heightfield, compact heightfield, distance field, regions, contours, and polygon mesh, then returns Zircon-friendly flat vertex/index/polygon/tile buffers

`src/ffi.rs` owns the Rust ABI declarations and C layout records, while `src/bake.rs` owns Rust-side bake input validation plus native-output conversion into `NavMeshAsset`. The upstream license is kept in `vendor/recastnavigation/License.txt`.

## Runtime Facade

The Rust facade still performs Zircon asset packaging and deterministic graph queries. It can bake simple fallback surfaces, rasterize collected triangle mesh input through native Recast into `.znavmesh` asset data with per-polygon areas, query connected polygons, apply area masks and area costs, reject mismatched agent-type queries, sample the nearest allowed triangle/edge instead of a broad asset AABB, raycast through sampled straight polygon visibility without treating off-mesh links as line-of-sight bridges, and include off-mesh links in route flags for path results.

`NavMeshAsset` carries copied area cost records from the active navigation settings. The backend uses those records to reject non-walkable areas and to weight polygon/link traversal, with link `cost_override` taking precedence when present. Binary `NavMeshAsset::to_bytes()` / `from_bytes()` round-trip tests protect deterministic `.znavmesh` artifact payloads shared with the runtime asset store.

The native bake boundary normalizes downward-wound triangles before slope filtering so runtime-collected quads and imported mesh data do not disappear solely because of winding. This split lets runtime/editor integration proceed while the native C ABI grows from Recast bake output toward Detour query objects, TileCache obstacle carving, and DetourCrowd simulation.

## Validation

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` passed: 14 unit tests and doctests. The tests cover native bridge linkage, simple-surface pathing, native Recast raster bake filtering for steep faces, area masks, disconnected islands, off-mesh link bridging, agent mismatch errors, deterministic binary roundtrip, nearest-polygon sampling, vertical projection, triangle-edge projection, and raycast behavior that ignores off-mesh links while reporting straight-line gaps as hits.
