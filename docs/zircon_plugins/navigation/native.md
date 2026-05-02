---
related_code:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
implementation_files:
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/lib.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --target-dir E:\cargo-targets\zircon-navigation-native-check --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Navigation Native Backend

## Purpose

`zircon_plugin_navigation_recast` is the native backend boundary for the navigation plugin. It vendors upstream Recast Navigation C++ sources and keeps the public Rust API expressed in Zircon DTOs (`NavMeshAsset`, `NavPathQuery`, `NavPathResult`, bake input records, and structured navigation errors).

## Native Boundary

`build.rs` compiles the vendored Recast, Detour, DetourCrowd, and DetourTileCache source folders plus `native/recast_bridge.cpp` through the `cc` crate. The C ABI currently exposes:

- bridge version reporting
- a smoke check that allocates/frees Detour navmesh, DetourCrowd, and DetourTileCache objects and calls a Recast bounds helper
- native polyline length calculation used by the Rust facade path result

The upstream license is kept in `vendor/recastnavigation/License.txt`.

## Runtime Facade

The Rust facade still performs Zircon asset packaging and deterministic graph queries. It can bake simple fallback surfaces, package collected triangle mesh input into `.znavmesh` asset data with per-triangle areas, query connected polygons, apply area masks, sample positions, raycast through query results, and include off-mesh links in route flags.

This split lets runtime/editor integration proceed while the native C ABI grows toward full Recast rasterization, Detour query objects, TileCache obstacle carving, and DetourCrowd simulation.

## Validation

The native crate check was attempted with an isolated target directory after adding the C++ bridge, but the command timed out while several unrelated Cargo/rustc jobs were active. Do not treat native compilation as proven until the listed check completes with exit code 0.
