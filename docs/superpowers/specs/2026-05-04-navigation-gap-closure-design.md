# Navigation Gap Closure Design

## Summary

Close the remaining deep navigation/pathfinding gaps by extending the existing plugin-owned Recast/Detour backend from transient single-query behavior into a layered runtime navigation system. The target state adds persistent tiled TileCache state, full native off-mesh link semantics, DetourCrowd-backed agent simulation, imported model triangle bake input, and large-world tile activation/rebuild scheduling while keeping public contracts in `zircon_runtime::core::framework::navigation` and concrete behavior inside `zircon_plugins/navigation`.

## Scope

This design covers:

- shared navigation DTO and `.znavmesh` asset extensions for tiled navmesh data, tile-cache build metadata, dirty/rebuild state, crowd reports, and imported-geometry bake diagnostics,
- native Recast/Detour C ABI additions for persistent TileCache worlds, off-mesh link costs, multi-tile query handles, and DetourCrowd update state,
- runtime navigation manager ownership for loaded navmesh runtime state, obstacle identity, dirty tile queues, active tile ranges, and crowd agent state,
- bake collection of real `ModelAsset` primitive vertex/index triangles through world transforms,
- milestone validation that separates native C++ evidence from Rust/Cargo evidence when unrelated renderer compile drift blocks navigation tests,
- module documentation and acceptance evidence updates for every material code slice.

This design does not cover:

- editor Navigation window or viewport overlay deepening beyond any documentation needed for new runtime-visible DTOs,
- 2D navigation,
- dynamic skinned/deforming mesh runtime baking,
- async worker-thread execution for tile rebuilds beyond a deterministic queue/scheduler contract,
- replacing the VM/plugin architecture or moving navigation into `zircon_runtime` as concrete runtime implementation.

## Architecture Ownership

`zircon_runtime::core::framework::navigation` remains the neutral contract layer. It owns serializable descriptors, manager traits, query/result DTOs, runtime stats, and structured errors. It must not own native Detour handles, runtime caches, `dtCrowd`, `dtTileCache`, or plugin-specific scheduling objects.

`zircon_runtime::asset::assets::navigation` owns persisted `.znavmesh` DTOs. It may store copied tile payload bytes, tile metadata, cache-layer metadata, source geometry fingerprints, and rebuild diagnostics. It must not store process-local native pointers or C++ ownership tokens.

`zircon_plugins/navigation/native` owns all Recast, Detour, TileCache, and Crowd concrete execution. It exposes opaque C handles through Rust RAII wrappers and returns copied Zircon DTOs. Native state is private to this crate and is recreated or updated from `NavMeshAsset` plus runtime obstacle/agent descriptors.

`zircon_plugins/navigation/runtime` owns runtime navigation behavior. `DefaultNavigationManager` remains the module manager surface, but its internal state splits into focused owners for loaded navmesh records, TileCache runtime records, obstacle tracking, active tile scheduling, bake source collection, and crowd simulation. Runtime callers continue to use `NavigationManager` rather than concrete native handles.

This keeps navigation aligned with the repository-wide architecture: optional navigation behavior stays plugin-owned, shared contracts stay neutral, runtime world authority stays in `zircon_runtime::scene`, and editor authoring state does not become authoritative runtime state.

## Reference Evidence

The design uses the current Zircon navigation implementation as the source of truth and checks it against mature engine precedents:

- `zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Include/DetourCrowd.h` shows the agent parameter, target request, and `dtCrowd::update(dt)` model needed for native crowd simulation.
- `zircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Include/DetourTileCache.h` and `DetourTileCacheBuilder.h` justify persistent obstacle refs, compressed tile layers, and mutable tile rebuild/update behavior.
- `dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMesh.h` and `NavigationData.h` justify active tile tracking, dirty-area rebuilds, and time-sliced build/update boundaries.
- `dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/CrowdManagerBase.h` justifies making crowd simulation a runtime manager service fed by world agents instead of embedding it in scene entities.
- `dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationSystem.h` justifies invoker-like active tile selection for large worlds.
- `dev/bevy/examples/gltf/query_gltf_primitives.rs` and current Zircon `ModelAsset`/`MeshVertex` DTOs justify reading imported primitive vertex/index payloads as explicit bake geometry rather than relying on scene-node footprint approximations.

## Current Baseline

The existing navigation stack already has a real foundation:

- `NavMeshAsset` stores vertices, indices, polygons, tiles, area costs, and off-mesh links.
- `NavigationManager` exposes bake, load, path, sample, raycast, agent tick, settings, and stats APIs.
- native Recast bakes triangle mesh input into Zircon `NavMeshAsset` values.
- native Detour owns private `dtNavMesh`/`dtNavMeshQuery` handles for path/sample/raycast where the asset is representable.
- native TileCache currently creates a transient single-tile cache per carved query and has WSL C++ smoke evidence.
- runtime manager collects navigation components, obstacles, and off-mesh links, then falls back to deterministic Rust graph behavior when the native bridge cannot represent a case.

The remaining gaps are the ones this design closes:

- TileCache state is rebuilt per query and not persisted or incrementally updated.
- DetourCrowd is vendored and compiled but not wrapped or used for agent simulation.
- native off-mesh link handling does not yet fully honor `cost_override` or TileCache/off-mesh combinations.
- bake collection for render meshes uses cube/mesh node footprints instead of real imported model primitive triangles.
- `override_tile_size` and tile metadata do not yet drive a persistent large-world tiled runtime.

## Data Model

`NavMeshAsset` should become an explicit tiled persisted asset while preserving old serialized assets through `serde(default)` field additions. The asset keeps top-level `vertices`, `indices`, and `polygons` for debug, fallback, and editor overlays, but each `NavMeshTileAsset` gains enough metadata to map polygons and payloads back to a tile:

- stable tile id and grid coordinate,
- world-space bounds and polygon/index ranges,
- build settings hash and source geometry hash,
- optional Detour tile payload bytes copied from native build output,
- optional TileCache compressed layer payload bytes and layer metadata,
- dirty/rebuild status for loaded runtime state serialization or diagnostics.

New persisted records should be added as neutral DTOs, not native ABI mirrors:

- `NavMeshTileCoordAsset` for `x`, `y`, and `layer`,
- `NavMeshTileBuildAsset` for cell size, cell height, walkable settings, border size, and tile size used to build the tile,
- `NavMeshTileCacheLayerAsset` for compressed layer bytes plus bounds and grid metadata,
- `NavMeshSourceGeometryAsset` or an equivalent compact fingerprint record for source model/primitive identity and transform hash,
- `NavMeshTileDirtyStateAsset` for `clean`, `dirty`, `rebuild_pending`, and `failed` diagnostics where persisted diagnostics are useful.

`NavigationRuntimeStats` should expand with loaded tile count, active tile count, pending dirty tiles, active crowd agents, and active TileCache obstacles. `NavAgentTickReport` should expand with crowd-updated count, link-traversal count, tile rebuild count, and diagnostics for agents rejected by crowd setup.

Public query DTOs do not need native handle fields. Runtime-specific obstacle and crowd descriptors may use stable entity ids or runtime obstacle ids, but C++ object refs remain plugin-private.

## Native Backend

The native backend should split by execution responsibility instead of growing one bridge file:

- `native/detour_query.cpp` continues to own static navmesh query creation, path, sample, and raycast.
- `native/detour_tile_cache.cpp` evolves from transient query helper into a persistent TileCache world owner with add/update/remove obstacle calls and dirty tile rebuild/update calls.
- a new `native/detour_crowd.cpp` owns `dtCrowd`, agent add/remove/update, target requests, velocity/position readback, and link/corner flags.
- shared Detour navmesh construction helpers should be extracted only when needed to avoid duplicating tile payload, off-mesh, and neighbor-building logic between static queries, TileCache, and Crowd.

The C ABI additions should stay opaque and copy-based:

- create/free persistent TileCache world from a tiled `NavMeshAsset` payload,
- add/update/remove box and capsule/cylinder obstacles and return stable native obstacle ids,
- update TileCache with a bounded tile-rebuild budget and return changed tile ids,
- query paths against the mutable TileCache navmesh,
- create/free Crowd world from a loaded navmesh query owner,
- sync agent descriptors, positions, velocities, area masks, avoidance quality, and priorities,
- request move targets and run `dtCrowd::update(dt)`,
- copy back agent position, velocity, status, current target, corner count, and off-mesh link flags.

Native off-mesh handling should move `cost_override` support out of the Rust fallback-only lane. The Detour bridge should embed off-mesh connections for representable links, preserve traversal area and directionality, and apply per-link cost override either through a custom virtual query filter or through explicit path post-cost evaluation that still lets Detour produce the corridor. TileCache and Crowd must use the same off-mesh construction so links do not disappear when obstacles or crowd simulation are enabled.

## Runtime Manager

`DefaultNavigationManager` should keep the public `NavigationManager` trait but split internal files before adding more behavior. `manager.rs` is already near the large-file threshold, so the implementation plan should introduce focused modules such as:

- `loaded_navmesh.rs` for handle allocation and loaded asset records,
- `tile_runtime.rs` for persistent TileCache state, obstacle refs, active tiles, dirty queues, and rebuild budgets,
- `crowd_runtime.rs` for world-agent sync, native crowd owner lifecycle, target requests, and tick readback,
- `bake_geometry.rs` for render/collider/model geometry collection,
- `off_mesh_runtime.rs` only if link endpoint update/traversal state grows beyond bake-time embedding.

Loaded navmesh records should contain the copied `NavMeshAsset`, a native static query handle when representable, a persistent TileCache state when cache payloads or runtime carving are enabled, and a Crowd state when crowd simulation is needed. This avoids rebuilding native state for every query and gives obstacle/crowd updates a stable owner.

Obstacle updates should use stable world entity ids and descriptor snapshots. Each tick compares current obstacle descriptors against the stored snapshot, applies add/update/remove calls to TileCache, marks affected tiles dirty, and runs a bounded rebuild/update stage. `move_threshold`, `time_to_stationary`, and `carve_only_stationary` should be honored in runtime state instead of only being inert descriptor fields.

Agent ticks should prefer Crowd when a loaded navmesh and native crowd owner are available. The manager syncs active `NavMeshAgent` components into native agents, requests targets for agents with destinations, runs one crowd update, writes transforms back for agents with `update_position`, and preserves the current conservative movement fallback only when there is no loaded navmesh or the native crowd owner cannot represent the asset. `avoidance_quality`, `priority`, `radius`, `height`, `speed`, `acceleration`, and `area_mask` become crowd params instead of only local heuristics.

## Bake Geometry

Render-mesh bake collection should use real imported model geometry when available. The collector should resolve a scene node's model/mesh asset reference through the runtime asset layer, read `ModelAsset.primitives[*].vertices` and `indices`, apply the node world transform, and append transformed triangles to `BakeGeometry`.

The first implementation should support static primitive geometry only. Vertices with skinning weights are still read at bind pose/static asset position; dynamic skinned mesh deformation remains outside this slice. If a render-mesh node references a model that cannot be resolved, the bake report should include a warning diagnostic and only fall back to footprint geometry when the node has no usable model payload. Missing imported geometry must not silently appear as a successful high-fidelity bake.

The bake report should distinguish source counts:

- primitive/model triangles collected,
- collider triangles collected,
- footprint fallback triangles collected,
- source nodes skipped because model payload was unavailable,
- triangles removed by modifiers or carving obstacles.

## Tiled Worlds

`NavMeshSurfaceDescriptor::override_tile_size` should become an active bake input. Recast bake output should partition source geometry into tiles, persist per-tile metadata, and allow runtime queries to load only active tiles into mutable native state.

Runtime active-tile selection should start with a deterministic invoker-like model:

- every active agent with a destination is an implicit navigation invoker,
- loaded navmesh bounds can also act as a global fallback invoker for small worlds,
- active radius and rebuild radius are derived from surface settings or conservative defaults,
- active tiles are the grid cells intersecting those radii,
- dirty tiles outside active ranges stay pending instead of rebuilding immediately.

The scheduler should be deterministic and budgeted. Each `tick_world_agents` or explicit maintenance call may rebuild a bounded number of dirty active tiles, then report counts in runtime stats and tick reports. This mirrors Unreal-style active/dirty tile separation without requiring async worker threads in the first slice.

## Error Handling

New errors should remain structured through `NavigationErrorKind` plus clear messages. Expected categories are:

- invalid tiled asset payload or mismatched tile metadata,
- native TileCache world creation failure,
- native Crowd world creation failure,
- native obstacle add/update/remove failure,
- model asset payload unavailable during render-mesh bake,
- query agent mismatch or unsupported agent settings,
- tile rebuild failure with tile id and coordinate in the diagnostic message.

Runtime path queries should not panic if a persistent native owner cannot be created. They should report the diagnostic and fall back only when the fallback preserves the same semantics. For example, falling back from Crowd to conservative movement is acceptable for unavailable native crowd support; falling back from carved TileCache to uncarved path should be reported because it changes obstacle semantics.

## Validation

Validation follows the repository milestone-first policy. Implementation slices may add tests and docs without running full Cargo loops after every file. Each milestone has a named testing stage with compile/build/test commands and a correction loop.

Focused expected commands use the shared external target directory:

- `rustfmt --edition 2021 --check` on touched navigation Rust files,
- `git diff --check --` on touched navigation/docs files,
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never`,
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never`,
- WSL native `g++ 11.4.0` smoke harnesses for TileCache/Crowd C ABI behavior when Windows C++ or unrelated Rust compile blockers prevent Cargo from reaching navigation tests.

Known validation risk: current focused Cargo navigation tests can stop in unrelated renderer compile drift under `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs` before navigation tests execute. Navigation acceptance must record that blocker exactly instead of claiming Rust tests passed. The navigation lane should not patch renderer code unless a navigation change directly owns the failing surface.

## Documentation

Material code changes must update the existing module docs instead of only adding comments:

- `docs/zircon_plugins/navigation/native.md` for native ABI, Detour, TileCache, Crowd, and validation evidence,
- `docs/zircon_plugins/navigation/runtime.md` for manager state, bake collection, runtime scheduling, crowd behavior, and stats,
- `docs/zircon_runtime/core/framework/navigation.md` if shared DTOs or manager trait contracts change,
- `docs/zircon_runtime/asset/assets/navigation.md` if `.znavmesh` persistence or tile/cache metadata changes,
- acceptance records under `tests/acceptance/` for TileCache persistence, DetourCrowd ticking, imported model bake geometry, and tiled-world dirty rebuilds.

Every affected document should keep its machine-readable related-code header current where that header exists.

## Milestones

### M0: Baseline And Coordination

Confirm active sessions, branch policy, target-dir space, current navigation validation blockers, and the current native/Rust evidence baseline. No production behavior changes are part of this milestone.

Testing stage: run only lightweight coordination and existing blocker confirmation commands. Record whether Cargo still stops before navigation tests.

### M1: Tiled Asset And DTO Foundation

Extend `NavMeshAsset`, tile metadata DTOs, runtime stats, tick reports, and serialization tests so tiled/cache metadata can exist without native owners. Preserve old asset deserialization through serde defaults.

Testing stage: focused `zircon_runtime` navigation asset/framework tests plus rustfmt and diff checks.

### M2: Persistent TileCache World

Replace per-query TileCache construction with a persistent native/Rust TileCache owner tied to a loaded navmesh record. Add obstacle add/update/remove, dirty tile marking, and bounded rebuild/update reporting.

Testing stage: native WSL TileCache harness, native Rust TileCache tests when Cargo reaches them, runtime obstacle-carving regression, and docs update evidence.

### M3: Native Off-Mesh Completion

Embed off-mesh links consistently across static Detour, TileCache, and Crowd-ready navmesh construction. Honor `cost_override`, traversal area, directionality, and path flags without forcing fallback for representable links.

Testing stage: off-mesh link path-cost ordering, one-way link rejection, TileCache plus off-mesh obstacle case, and no-path cases for masked link areas.

### M4: Imported Model Geometry Bake Input

Resolve `ModelAsset` primitive vertex/index payloads for render-mesh bake collection, apply world transforms, emit detailed diagnostics, and keep collider/footprint fallbacks explicit.

Testing stage: runtime bake tests proving imported primitive triangles drive source counts and baked output; diagnostics for missing model payload; rustfmt/diff checks.

### M5: DetourCrowd Runtime Agents

Add native Crowd world ownership and runtime sync from `NavMeshAgent` components. Use Crowd for loaded-navmesh agent motion and keep conservative movement as a reported fallback.

Testing stage: native Crowd smoke harness, runtime tests for two agents avoiding each other, destination update, disabled `update_position`, no loaded navmesh fallback, and stats/report counters.

### M6: Large-World Tile Activation And Rebuild Scheduling

Make `override_tile_size` drive tile partitioning and runtime active tile selection. Add deterministic active/dirty tile queues and bounded rebuild budgets.

Testing stage: multi-tile bake persistence, active tile selection around agents, dirty inactive tile deferral, dirty active tile rebuild, and query behavior across tile boundaries.

### M7: Acceptance And Gap Report

Update native/runtime/framework/asset docs, add acceptance records, rerun focused validation, and produce a final residual-gap report that distinguishes closed gaps, externally blocked validation, and deliberately out-of-scope future work.

Testing stage: focused navigation test matrix plus WSL native evidence where needed. Broad workspace validation is only claimed if it actually runs successfully.

## Acceptance Criteria

The work is accepted when:

- loaded navmesh records reuse persistent native query/TileCache/Crowd owners instead of rebuilding everything per query,
- carving obstacles update incrementally and affect path queries through dirty tile rebuilds,
- off-mesh links with `cost_override` work through native representable query paths,
- runtime agent movement uses DetourCrowd when a loaded navmesh supports it,
- imported model primitive triangles can feed Recast bake input with diagnostics for missing payloads,
- tiled assets persist tile/cache metadata and runtime queries can operate across active tiles,
- docs and acceptance evidence identify commands run, blockers seen, and remaining risks,
- unrelated renderer compile drift is reported as an external validation blocker, not hidden as navigation success.
