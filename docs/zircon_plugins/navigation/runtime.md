---
related_code:
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/components/agent.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/agent/repath.rs
  - zircon_plugins/navigation/runtime/src/agent/writeback.rs
  - zircon_plugins/navigation/runtime/src/components/modifier.rs
  - zircon_plugins/navigation/runtime/src/components/obstacle.rs
  - zircon_plugins/navigation/runtime/src/components/off_mesh_bridge.rs
  - zircon_plugins/navigation/runtime/src/components/off_mesh_link.rs
  - zircon_plugins/navigation/runtime/src/components/surface.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/manager/agent_motion.rs
  - zircon_plugins/navigation/runtime/src/manager/bake.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/area_volume.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/asset.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/diagnostics.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/dirty.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/filter.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/geometry.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/modifier.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/source_selection.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/surface.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/tiled.rs
  - zircon_plugins/navigation/runtime/src/manager/query.rs
  - zircon_plugins/navigation/runtime/src/manager/state.rs
  - zircon_plugins/navigation/runtime/src/manager/stats.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/navigation/runtime/src/manager/traversal.rs
  - zircon_plugins/navigation/runtime/src/component_json.rs
  - zircon_plugins/navigation/runtime/src/off_mesh_connections.rs
  - zircon_plugins/navigation/runtime/src/runtime_obstacles.rs
  - zircon_plugins/navigation/runtime/src/settings_hash.rs
  - zircon_plugins/navigation/runtime/src/settings_validation.rs
  - zircon_plugins/navigation/runtime/src/tests/mod.rs
  - zircon_plugins/navigation/runtime/src/tests/bake.rs
  - zircon_plugins/navigation/runtime/src/tests/crowd.rs
  - zircon_plugins/navigation/runtime/src/tests/tiled_bake_context.rs
  - zircon_plugins/navigation/runtime/src/tests/dynamic_components.rs
  - zircon_plugins/navigation/runtime/src/tests/manager.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
  - zircon_plugins/navigation/runtime/src/tests/support.rs
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
  - zircon_runtime/src/core/framework/navigation/manager.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime/src/core/framework/navigation/manager.rs
  - zircon_runtime/src/core/framework/navigation/query.rs
  - zircon_runtime/src/asset/artifact/store.rs
implementation_files:
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_plugins/navigation/runtime/src/components/agent.rs
  - zircon_plugins/navigation/runtime/src/components/modifier.rs
  - zircon_plugins/navigation/runtime/src/components/obstacle.rs
  - zircon_plugins/navigation/runtime/src/components/off_mesh_bridge.rs
  - zircon_plugins/navigation/runtime/src/components/off_mesh_link.rs
  - zircon_plugins/navigation/runtime/src/components/surface.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_plugins/navigation/runtime/src/manager/agent_motion.rs
  - zircon_plugins/navigation/runtime/src/manager/bake.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/area_volume.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/asset.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/diagnostics.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/dirty.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/filter.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/geometry.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/modifier.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/source_selection.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/surface.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/tiled.rs
  - zircon_plugins/navigation/runtime/src/manager/query.rs
  - zircon_plugins/navigation/runtime/src/manager/state.rs
  - zircon_plugins/navigation/runtime/src/manager/stats.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/navigation/runtime/src/manager/traversal.rs
  - zircon_plugins/navigation/runtime/src/component_json.rs
  - zircon_plugins/navigation/runtime/src/off_mesh_connections.rs
  - zircon_plugins/navigation/runtime/src/runtime_obstacles.rs
  - zircon_plugins/navigation/runtime/src/settings_hash.rs
  - zircon_plugins/navigation/runtime/src/settings_validation.rs
  - zircon_plugins/navigation/runtime/src/tests/mod.rs
  - zircon_plugins/navigation/runtime/src/tests/bake.rs
  - zircon_plugins/navigation/runtime/src/tests/tiled_bake_context.rs
  - zircon_plugins/navigation/runtime/src/tests/dynamic_components.rs
  - zircon_plugins/navigation/runtime/src/tests/manager.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
  - zircon_plugins/navigation/runtime/src/tests/support.rs
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
  - user: 2026-05-10 retained-host workspace validation handoff: navigation runtime world scans
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/sessions/20260510-0050-navigation-runtime-world-scan.md
tests:
  - navigation_registration_contributes_runtime_module_and_components
  - navigation_module_resolves_manager_and_queries_loaded_navmesh
  - navigation_dynamic_component_descriptor_accepts_vec_and_resource_json
  - bake_surface_expands_offmesh_bridge_lanes_and_tracks_stats
  - tile_bake_matches_simple_bake_geometry
  - tiled_bake_does_not_block_main_thread
  - tile_boundary_paths_are_continuous
  - dirty_tile_rebuild_only_affects_neighbors
  - dirty_tile_rebuild_reconciles_vacated_and_new_tiles
  - dirty_tile_rebuild_rejects_changed_bake_identity
  - dirty_tile_rebuild_can_empty_the_entire_previous_grid
  - successful_non_tiled_bake_clears_previous_tiled_snapshot
  - stale_async_bake_cannot_restore_snapshot_after_newer_full_bake
  - settings_update_retires_inflight_bake_and_snapshot
  - newer_bake_retires_superseded_full_and_dirty_tasks
  - tiled_snapshots_are_isolated_per_explicit_surface
  - default_and_explicit_selection_share_the_same_surface_context
  - concurrent_same_surface_submissions_atomically_retire_the_older_handle
  - navigation_manager_limits_agent_start_velocity_by_acceleration
  - navigation_manager_auto_braking_stops_at_agent_stopping_distance
  - automatic_agent_tick_does_not_cross_manual_off_mesh_links
  - automatic_agent_tick_respects_auto_traverse_links_opt_out
  - explicit_path_query_can_still_cross_manual_off_mesh_links
  - navigation_plugin_toml_matches_catalog_beta_partial_metadata
  - cargo test --manifest-path zircon_plugins\navigation\runtime\Cargo.toml navigation_registration_contributes_runtime_module_and_components --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-navigation-runtime-metadata --color never --quiet
  - cargo test --manifest-path Cargo.toml -p zircon_runtime --lib navigation_plugin_toml_matches_catalog_beta_partial_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-navigation-runtime-metadata --color never --quiet
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
  - cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never
  - wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER ... zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp ... -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke"
  - cargo test --manifest-path zircon_plugins\Cargo.toml --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never
  - cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime --lib carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never -- --nocapture
  - 2026-06-01: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --message-format short --color never (passed 13 tests after package manifest option/event strictness fix)
  - rustfmt --edition 2021 --check zircon_plugins/navigation/runtime/src/manager/bake.rs zircon_plugins/navigation/runtime/src/manager/bake/asset.rs zircon_plugins/navigation/runtime/src/manager/bake/diagnostics.rs zircon_plugins/navigation/runtime/src/manager/bake/filter.rs zircon_plugins/navigation/runtime/src/manager/bake/geometry.rs zircon_plugins/navigation/runtime/src/manager/bake/modifier.rs zircon_plugins/navigation/runtime/src/manager/bake/surface.rs (2026-06-04 bake boundary split: passed)
  - git diff --check -- zircon_plugins/navigation/runtime/src/manager/bake.rs zircon_plugins/navigation/runtime/src/manager/bake docs/zircon_plugins/navigation/runtime.md docs/zircon_runtime/core/framework/navigation.md (2026-06-04 bake boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-bake-split-0604 --message-format short --color never (planned for current bake boundary split)
  - rustfmt --edition 2021 --check zircon_plugins/navigation/runtime/src/components.rs zircon_plugins/navigation/runtime/src/components/agent.rs zircon_plugins/navigation/runtime/src/components/modifier.rs zircon_plugins/navigation/runtime/src/components/obstacle.rs zircon_plugins/navigation/runtime/src/components/off_mesh_bridge.rs zircon_plugins/navigation/runtime/src/components/off_mesh_link.rs zircon_plugins/navigation/runtime/src/components/surface.rs (2026-06-04 component descriptor boundary split: passed)
  - git diff --check -- zircon_plugins/navigation/runtime/src/components.rs zircon_plugins/navigation/runtime/src/components docs/zircon_plugins/navigation/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 component descriptor boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-components-split-0604 --message-format short --color never (planned for current component descriptor boundary split)
  - cargo test -p zircon_runtime --lib world_mutations_mark_derived_state_dirty_until_post_update_systems_flush --locked --target-dir target\codex-shared-b -- --nocapture --test-threads=1
  - cargo test -p zircon_plugin_navigation_runtime --lib --locked --target-dir target\codex-shared-b -- --nocapture --test-threads=1
doc_type: module-detail
---

# Navigation Runtime Plugin

## Purpose

The navigation runtime plugin owns the first real navigation manager surface for Zircon. It embeds the canonical `navigation.runtime` descriptor, contributes six dynamic component descriptors, loads baked navmesh assets, exposes path/sample/raycast APIs through the shared `NavigationManager` trait, and ticks basic scene agents toward destinations.

## Related Files

`src/lib.rs` wires the plugin descriptor, module descriptor, package manifest helpers, and extension registration. It now stays structural and delegates test coverage to `src/tests/`. `src/components.rs` is the structural descriptor list for the six navigation authoring components, while `components/{surface,modifier,agent,obstacle,off_mesh_link,off_mesh_bridge}.rs` own each descriptor's editable property schema. `src/manager.rs` is now the structural `DefaultNavigationManager` facade: it owns the backend/state lock, navigation bake task pool, service trait implementation, manager state mutation, and delegating entry points. `manager/bake.rs` is now the bake orchestration facade: `bake/surface.rs` owns enabled-surface discovery and request selection, `bake/geometry.rs` owns render/collider footprint collection and source counters, `bake/filter.rs` owns collection-mode/layer/volume filtering, `bake/modifier.rs` owns direct and inherited `NavMeshModifier` lookup, `bake/diagnostics.rs` owns bake diagnostics, and `bake/asset.rs` owns Recast/simple fallback dispatch, settings stamping, and off-mesh link embedding. M2 keeps tile concerns separated: `bake/tiled.rs` maps prepared geometry to a stable Recast tile plan, `bake/task_pool.rs` owns non-blocking parallel tile dispatch and main-thread harvesting, and `bake/dirty.rs` owns dirty-AABB neighborhood selection, previous/current world-grid reconciliation, bake-identity validation, and unaffected-tile preservation. `manager/query.rs` owns loaded-asset path/sample/raycast forwarding; `manager/agent_motion.rs` owns manager-private per-entity velocity state, acceleration-limited velocity planning, auto-braking travel clamps, and angular-speed-limited yaw updates; `manager/traversal.rs` owns the automatic-agent off-mesh traversal policy that filters manual links or all links for agents that opt out before backend path queries; `manager/tick.rs` owns runtime agent scanning, path-following, local avoidance, transform writeback, and tick stats updates; `manager/state.rs` owns loaded navmesh handles/settings/stats/agent-motion state plus pending and identity-bound last-completed tiled bake state; and `manager/stats.rs` owns immediate dynamic-component counting. `src/component_json.rs` normalizes tagged editor property JSON before component deserialization; `src/off_mesh_connections.rs` collects active off-mesh links and expands off-mesh bridges into bounded per-lane baked links; `src/runtime_obstacles.rs` collects runtime obstacle descriptors and converts carving obstacles into Recast-native obstacle records; `src/settings_hash.rs` and `src/settings_validation.rs` keep stable bake-hash and settings-validation logic out of the manager files.

`src/tests/mod.rs` is the structural runtime test entry. `tests/registration.rs` covers plugin/module/component/option/event manifest contribution, `tests/manager.rs` covers manager resolution, loaded-navmesh queries, acceleration-limited agent ticking, auto-braking arrival clamps, obstacle avoidance, no-path blocking, handle selection, and settings validation, `tests/dynamic_components.rs` covers dynamic component JSON property normalization, `tests/bake.rs` covers surface geometry, tiled/dirty bake behavior, modifier/off-mesh-link embedding, settings hashes, link-generation disabling, and carving, `tests/tiled_bake_context.rs` owns per-surface generation, settings invalidation, superseded-task retirement, default/explicit aliasing, and concurrent submission coverage, and `tests/support.rs` holds shared navmesh fixtures.

`navigation/native` is the Recast/Detour backend boundary. It vendors upstream Recast Navigation C++ sources for Recast, Detour, DetourCrowd, and DetourTileCache under `vendor/recastnavigation`, compiles them through `cc`, and exposes C ABI bridge files under `native/`. `src/bake.rs` validates bake input, creates stable tile-grid plans, shares prepared mesh/tile/native input buffers across task clones, merges tile-local vertex/index buffers, and converts native Recast bake buffers into `NavMeshAsset`; `src/detour.rs` wraps an opaque native `dtNavMesh` / `dtNavMeshQuery` owner for path/sample/raycast queries; `src/tile_cache.rs` wraps an opaque native TileCache query owner for carved obstacle path queries; `src/ffi.rs` owns the ABI declarations. Triangle-mesh and `zr_nav_recast_bake_tile` calls pass through Recast heightfield, bordered region, contour, and polygon mesh construction before returning Zircon DTOs. Tile merge deduplicates quantized boundary vertices so representable multi-tile assets remain connected when converted into the current Detour query owner.

## Behavior Model

Registration contributes:

- `navigation.runtime` with lazy manager `navigation.runtime.Manager.NavigationManager`
- runtime capabilities `runtime.plugin.navigation` and `runtime.plugin.navigation.recast`
- package metadata category `runtime`, maturity `beta`, and `runtime.plugin.navigation = partial` status with an explicit note that gameplay navmesh/pathfinding is separate from Bevy-style UI navigation parity
- the six `navigation.Component.*` dynamic component descriptors
- plugin options for the default agent type, default settings asset, debug gizmos, and bake backend
- event catalog entries for bake completion, path query completion/failure, and agent ticks

The package manifest uses the shared strict manifest contract. `navigation.default_settings_asset` is a non-empty string defaulting to `res://navigation/settings/default.navigation.toml`; `navigation.bake_backend` is an enum whose only current value is `recast`. The event catalog namespace is `navigation.runtime`, every event id is kept under that namespace, and every payload schema is lowercase, package-prefixed, and versioned, for example `navigation.runtime.navmesh_bake_report.v1`.

`DefaultNavigationManager` keeps loaded `NavMeshAsset` values in a mutex-protected map and returns stable `NavMeshHandle` values. Queries can address a specific handle or fall back deterministically to the lowest loaded handle. `find_path_with_filter` accepts caller-owned `NavQueryFilter` costs and include/exclude polygon flags. Empty maps return a structured missing-navmesh error; empty assets or blocked area masks/flags return `NoPath`.

The current backend supports deterministic simple-surface fallback baking, native Recast raster/poly-mesh baking for collected triangle geometry, stable settings-hash stamping, area-cost copying, Detour-backed path/sample/raycast queries for representable assets, persistent per-navmesh DetourTileCache obstacle carving, Rust polygon-graph fallback queries for unsupported asset shapes or off-mesh cost-override cases, area-mask filtering, disconnected-island no-path results, optional off-mesh connection embedding controlled by `NavMeshSurface.generate_links`, single off-mesh links that bridge otherwise disconnected polygons, and wider off-mesh bridges that expand to bounded per-lane baked links. Setting `NavMeshSurface.override_tile_size` now selects a world-unit tile grid. Synchronous `bake_surface` produces the merged tiled asset, while `start_tiled_bake` takes ownership of a world snapshot and performs geometry collection, tile planning, and each tile bake on the runtime async-compute pool; `bake_task_state` and `try_harvest_tiled_bake` let the caller poll and harvest without joining worker threads on the main thread. Completed bake diagnostics are retained in the manager diagnostic store. `start_dirty_tile_rebuild` and `try_harvest_dirty_tile_rebuild` apply the same submit/poll/harvest model to a finite dirty AABB: the background work expands it by one tile, reconciles previous/current tile bounds, rebuilds intersecting tile ids including newly occupied, vacated, or fully emptied tiles, and preserves byte-equivalent polygon geometry elsewhere. The cached source is bound to surface entity, agent, full surface descriptor, settings, and tile size; identity changes require an explicit full rebuild instead of mixing unrelated polygons. Each actually selected surface owns an independent monotonic generation and tiled snapshot; default and explicit selection of the same surface are canonicalized to that same context. Only the latest generation in a context may publish snapshots, diagnostics, or bake-derived stats, so late harvest cannot roll state backward or erase another surface's dirty-rebuild baseline. Superseded results return a typed error, settings changes invalidate all contexts, abandoned handles are retired on the next same-context request, and task/manager mutex poison is recovered at the centralized lock boundary. A successful non-tiled bake clears only its context's tiled snapshot. Bake input continues to honor the M1 physics-first policy: a physics-collider request that yields no geometry and was not intentionally emptied by modifiers or carving is recollected from render meshes and emits an explicit fallback diagnostic. Empty-node modifier volumes apply their area id to source geometry whose world-space center lies inside the volume before Recast polygonization. Scene authoring scans use fresh `World::node_records()` projections instead of the deferred `World::nodes()` cache so dynamic navigation components attached through direct world mutation are visible before the next scheduled `PostUpdate` cache refresh. Agent ticking reads `NavMeshAgent` dynamic components, follows an optional `destination`, prefers the next path waypoint from the loaded navmesh when available, falls back to direct movement only when no navmesh is loaded, blocks and reports agents when a loaded navmesh returns no path or an invalid query, respects `speed`, `acceleration`, `stopping_distance`, `auto_braking`, `angular_speed`, `update_position`, `update_rotation`, and `auto_traverse_links`, performs basic obstacle/agent separation, and updates the entity transform through `World::update_transform`. Manual off-mesh links remain in baked assets and explicit `find_path` results, but automatic agent ticks filter them out until a gameplay system explicitly handles manual traversal.

## Design and Rationale

The plugin keeps navigation behavior out of `zircon_runtime` while still making the manager visible through the existing module/service system. This follows the independent-plugin direction and lets editor, scripting, and future native backends use one neutral API.

The native C++ boundary is intentionally narrow. It builds upstream Recast/Detour/DetourCrowd/DetourTileCache as part of the plugin without leaking native handles into shared DTOs. Recast rasterization backs triangle-mesh bakes, opaque Detour query owners back normal runtime queries, TileCache backs carved obstacle path queries, and persistent DetourCrowd owners now back loaded-navmesh agent simulation.

Agent motion follows the same separation. `NavMeshAgentDescriptor` stays a serializable authoring/configuration DTO in `zircon_runtime::core::framework::navigation`; the concrete plugin owns per-entity velocity as manager state and owns the policy for whether automatic movement may consume off-mesh links. That mirrors Recast DetourCrowd's distinction between agent params and runtime velocity, Godot's split between navigation agents and enabled navigation links, and Unreal's separation between simple `NavLink` traversal and custom link gameplay handoffs without exposing those backend objects in the shared framework.

## Control Flow

At plugin registration time, `NavigationRuntimePlugin::register` registers the module and every component descriptor. `load_navigation_settings` validates unique agent/area ids, finite numeric settings, non-empty names, and maskable area ids before installing settings. When the manager bakes a surface, it scans fresh scene node projections for enabled `NavMeshSurface` descriptors, chooses the requested surface or the first enabled surface, validates the requested agent type against `NavigationSettingsAsset`, and collects bake geometry according to the surface collection mode. Render-mesh mode uses cube/mesh node footprints when model vertex payloads are not available through the world; collider mode uses box/sphere/capsule collider footprints. The collector excludes navigation surface authoring volumes, agents, obstacles, off-mesh-link nodes, and off-mesh-bridge nodes from source geometry; applies nearest `NavMeshModifier` remove/area override rules; treats a modifier on the selected surface as a bake-scope area override; removes static bake sources intersecting carving obstacles; then sends remaining triangles through native Recast rasterization. The native boundary normalizes downward-wound triangles before slope filtering, builds Recast regions/contours/polygons, and returns tile/polygon buffers. The manager optionally embeds active off-mesh connections, stamps an explicit stable FNV-style hash over surface/settings fields, copies area costs into the asset, and returns a `NavMeshBakeReport` with source counts and diagnostics.

At runtime query time, `load_nav_mesh` stores the asset, `find_path` delegates to the backend, `sample_position` finds the nearest allowed polygon sample inside query extents, and `raycast` reports clear or blocked traversal. Explicit path queries use the stored asset as authored, including manual off-mesh links, because callers may be planning their own traversal handoff. Automatic agent ticks first ask `manager/traversal.rs` for a query view: agents with `auto_traverse_links = false` query without off-mesh links, and other agents query only automatic links. The backend then builds a native Detour query owner from copied asset buffers, reconstructs shared-edge neighbour data, applies Zircon area masks plus caller filter costs/flags through a virtual Detour filter, and uses `findPath` / `findStraightPath`, `findNearestPoly`, or `raycast` depending on the request. When the world contains active carving obstacles, `runtime_obstacles.rs` creates one `NavigationObstacleWorld` per navmesh, retains its `RecastTileCache`, maps obstacle entity ids to native handles, applies add/remove/change deltas, and updates the native navmesh before querying. Removing the component drives `removeObstacle` on the same cache and restores the corridor on the next tick. When the asset cannot be represented by the Detour bridge, the backend falls back to the lightweight Rust polygon graph and sampled visibility path, preserving off-mesh `cost_override` behavior and previous no-path semantics. Agent ticking is intentionally conservative and mutates only dynamic entities whose navigation component declares a destination. Each active agent gets a manager-private velocity cache keyed by entity id; the cache is pruned when the agent disappears, has no destination, disables position updates, reaches stopping distance, or hits a blocking query/update failure. Runtime stats track loaded navmeshes, scanned active agents, active obstacles, active off-mesh links, and active off-mesh bridges. Component parsing normalizes editor property tags such as `{ "resource": "..." }` and `{ "entity": 12 }` before deserializing navigation descriptors.

## Edge Cases

Agent movement can be blocked by missing transforms, immutable/static entity transforms, manual-only off-mesh links, or an agent-level `auto_traverse_links = false` path that would otherwise require a link, and those failures are reported in `NavAgentTickReport`. Non-finite or negative speed, acceleration, angular speed, and stopping distance values are clamped by the runtime motion path instead of producing NaN transforms, while settings assets still reject invalid global agent definitions before installation. Obstacle support is intentionally scoped: bake-time carving removes intersecting collected static sources before rasterization, runtime path queries keep persistent per-navmesh TileCache state and incrementally synchronize component changes, and runtime avoidance applies simple local separation from obstacle centers and neighboring agents. The manager does not yet persist compressed TileCache layers in `NavMeshAsset` or converge off-mesh traversal and dynamic carving into the persistent Crowd lifecycle; those scenes deliberately stay on the isolated legacy movement path. Crowd simulation, per-navmesh ownership, area-filtered corridors, repath budgets, and Transform/DesiredVelocity writeback are implemented.

## Test Coverage

2026-07-12 M1 SimpleBake validation added the exact plan anchors `bake_input_falls_back_to_render_mesh_without_physics`, `golden_level_bake_then_path_length_within_tolerance`, and `modifier_volume_marks_area_id_in_polymesh`. A pinned-HEAD Windows validation copy, overlaid only with Navigation M1 sources, passed the runtime package with 24 tests and 0 failures and the native Recast package with 20 tests and 0 failures. Shared-checkout validation remains a separate closeout gate because concurrent Frameworks asset-codec work currently prevents unrelated packages from compiling.

2026-07-12 final M4 Windows validation passed native job `5c1a96ab19e54cb1bb47d091979e17d7` (31 unit + 4 integration + doctests) and runtime job `567fc95691f44ec9a43ca895aabcfcc3` (50 unit + doctests). Exact anchors include carve/remove/restore, 65-request incremental/batch queue flushing, cache-scoped handles, remove-before-replacement slot release, service-handle filtered queries, baked-cost preservation, native/fallback area-cost direction parity, and flag exclusion.

2026-05-31 linked metadata parity first failed `cargo test --manifest-path zircon_plugins\navigation\runtime\Cargo.toml navigation_registration_contributes_runtime_module_and_components --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-navigation-runtime-metadata --color never --quiet` because the linked package manifest still reported `Experimental` instead of `Beta`. After adding the linked descriptor maturity/status metadata and static `plugin.toml` category, the same command passed with 1 Navigation runtime test and 0 failures. `cargo test --manifest-path Cargo.toml -p zircon_runtime --lib navigation_plugin_toml_matches_catalog_beta_partial_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-navigation-runtime-metadata --color never --quiet` also passed with 1 static TOML/catalog test and 0 failures. Existing output was limited to unrelated `zircon_runtime` warnings.

Prior to the TileCache slice, `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` passed: 12 unit tests and doctests. The tests cover runtime registration, dynamic component descriptor JSON conversion, typed resource properties, native Recast-backed surface baking, bake modifier/off-mesh-link embedding, link-generation disablement and settings hash stamping, obstacle carving, basic obstacle avoidance/stats, path queries over loaded navmeshes, deterministic default mesh selection, loaded-navmesh no-path agent blocking, invalid settings rejection, and agent ticking.

2026-06-04 structural test split moved the inline `src/lib.rs` test block into `src/tests/{mod,registration,manager,dynamic_components,bake,support}.rs`. `rustfmt --edition 2021 --check --config skip_children=true zircon_plugins/navigation/runtime/src/lib.rs zircon_plugins/navigation/runtime/src/tests/mod.rs zircon_plugins/navigation/runtime/src/tests/support.rs zircon_plugins/navigation/runtime/src/tests/registration.rs zircon_plugins/navigation/runtime/src/tests/manager.rs zircon_plugins/navigation/runtime/src/tests/dynamic_components.rs zircon_plugins/navigation/runtime/src/tests/bake.rs` passed for the moved root/test files. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running, so this structural split is not yet test-compile accepted.

2026-06-04 manager boundary split reduced `zircon_plugins/navigation/runtime/src/manager.rs` from an 855-line mixed manager into a 138-line facade plus `manager/{bake,query,state,stats,tick}.rs`. The split follows the NavigationSystem/NavMesh/agent separation visible in Unreal's NavigationSystem tree and keeps Recast bake/query, ECS world scans, agent movement, and state accounting in separate Rust modules. Static validation evidence is tracked in the live session note; focused Cargo validation remains pending while other active Cargo/rustc lanes are running.

2026-06-04 bake boundary split reduced `zircon_plugins/navigation/runtime/src/manager/bake.rs` from a 509-line mixed bake implementation into a 106-line facade plus `manager/bake/{asset,diagnostics,filter,geometry,modifier,surface}.rs`. The split follows Unreal's NavMesh/Recast/NavLink separation and Godot's navigation region/link/obstacle/server/generator boundaries while preserving the current bake request, geometry collection, modifier/obstacle filtering, off-mesh embedding, diagnostics, and report behavior. Static validation evidence is tracked in the live session note; focused Cargo validation remains pending while other active Cargo/rustc lanes are running.

2026-06-04 component descriptor boundary split reduced `zircon_plugins/navigation/runtime/src/components.rs` from a mixed six-descriptor declaration file into a structural descriptor list plus folder-backed `components/{surface,modifier,agent,obstacle,off_mesh_link,off_mesh_bridge}.rs`. The split preserves descriptor order, component type ids, plugin id metadata, property names, property kinds, and required flags while making each authoring component's schema independently owned. Static validation evidence is tracked in the live session note; focused Cargo validation remains pending while other active Cargo/rustc lanes are running.

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never` also passed before the TileCache slice with 16 unit tests, 3 Detour integration tests, and doctests, including native Recast raster bake filtering for steep faces, non-finite source rejection before FFI, unique-vertex polygon adjacency for triangulated fan output, Detour string-pulled corridor paths, Detour sample projection, and Detour raycast boundary hits. `cargo check -p zircon_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-runtime-check --message-format short --color never` passed previously with existing graphics/UI warnings.

2026-05-07 M8 closeout reran plugin validation on `D:\cargo-targets\zircon-plugins-m8-final`. Full plugin workspace `cargo test --manifest-path zircon_plugins\Cargo.toml --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never` exited 0. During the rerun, `carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh` exposed a fixture issue rather than a manager issue: the old box obstacle did not span the full depth of `NavMeshAsset::simple_quad("humanoid", 3.0)`, leaving a valid detour. The fixture now spans the full simple-quad depth, and the focused rerun `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime --lib carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never -- --nocapture` passed: 1 passed / 0 failed.

Fresh 2026-05-10 workspace-blocker evidence traced the original retained-host validation handoff to navigation runtime scans reading `World::nodes()` before the `PostUpdate` node-cache refresh. The lower shared ECS contract stayed intact: `World::nodes()` remains schedule-maintained cached state, while `World::node_records()` projects current direct records for authoring systems that need immediate dynamic component visibility. The supporting ECS regression `cargo test -p zircon_runtime --lib world_mutations_mark_derived_state_dirty_until_post_update_systems_flush --locked --target-dir target\codex-shared-b -- --nocapture --test-threads=1` passed with `1 passed; 0 failed; 1195 filtered out`, and the focused navigation rerun `cargo test -p zircon_plugin_navigation_runtime --lib --locked --target-dir target\codex-shared-b -- --nocapture --test-threads=1` passed with `13 passed; 0 failed` after `collect_surfaces`, `collect_agents`, bake geometry, off-mesh-link scans, obstacle counts, navigation component stats, and runtime obstacle collection moved to fresh node projections. The broader `.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir target\codex-shared-b` rerun then passed `cargo build --workspace --locked` and moved the workspace blocker to `zircon_editor` lib-test compilation, where stale retained-host imports of deleted `primitives::Model` remain outside the navigation runtime module.

2026-06-01 package-manifest validation reran `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --message-format short --color never` under the M6 shared target after the shared registry began enforcing non-empty option defaults, enum value lists, dot-namespaced event catalogs, namespace-prefixed event ids, and versioned payload schemas. The command passed with 13 tests and 0 failures. The fix was limited to package metadata and its registration assertions; navigation baking, query, and agent behavior stayed unchanged.

2026-06-07 agent-motion slice added `manager/agent_motion.rs` as the runtime-only owner for per-entity velocity, acceleration-limited speed changes, arrival braking, and angular-speed-limited yaw updates. `tests/manager.rs` now covers `navigation_manager_limits_agent_start_velocity_by_acceleration` and `navigation_manager_auto_braking_stops_at_agent_stopping_distance` through the real `World` dynamic component plus `tick_world_agents` path. Static validation and focused Cargo evidence for this slice are tracked in the active plugin-ecosystem session note.

2026-06-07 automatic traversal-policy slice added `manager/traversal.rs` as the runtime-only owner for filtering off-mesh links before automatic agent path queries. `tests/manager.rs` now covers `automatic_agent_tick_does_not_cross_manual_off_mesh_links`, `automatic_agent_tick_respects_auto_traverse_links_opt_out`, and `explicit_path_query_can_still_cross_manual_off_mesh_links`, proving that manual links stay in explicit path planning while automatic agent movement does not silently cross them. Static validation evidence is tracked in the active plugin-ecosystem session note.

## DetourCrowd Agent Runtime

M3 replaces loaded-navmesh agent steering with one persistent DetourCrowd per `NavMeshHandle`. `NavMeshAgentDescriptor.nav_mesh` optionally selects the surface owner and otherwise uses the first loaded handle. `agent.rs` groups agents by handle, reconciles entity/native bindings, performs one native `update` plus one batch state read per active Crowd, and drops owners for unloaded handles. Parameter changes recreate only that native agent. `DesiredVelocity` agents feed their controller-owned Transform back through the native corridor-position synchronization API every frame, preventing hidden Crowd position drift. Until later milestones converge obstacle/off-mesh native lifecycles, scenes containing runtime obstacles or off-mesh links remain on the existing isolated manager path.

The runtime plugin declares and registers the `navigation.agent_tick` system anchor in `SystemStage::Update` after `ai.behavior_tick`. `NavRepathBudget` is a registered ECS resource; destination changes and blocked `auto_repath` retries consume at most `max_queries_per_frame`. Per-Crowd entity cursors plus a global handle cursor prevent starvation both within and across navmeshes. A budget unit means one actual Crowd `requestMoveTarget`; the prior transient preflight query was removed. Agent add/sync failures are isolated to that entity and reported without aborting other Crowds. `NavMeshAgentDescriptor.writeback_mode` chooses between direct Transform/rotation ownership and the registered `navigation.Component.NavDesiredVelocity` component for character controllers. Partial corridors and failed Crowd targets are reported as blocked instead of being written back as movement. The fresh validator job `de1d93af6e734c9d9af4eda1fb58d737` passed all 47 runtime package tests.
