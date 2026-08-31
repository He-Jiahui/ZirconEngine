---
title: Navigation Bake Recast Detour Native Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/navigation/runtime/src/manager/bake.rs
  - zircon_plugins/navigation/runtime/src/manager/bake
  - zircon_plugins/navigation/native
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavigationDirtyAreasController.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavigationSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavMesh/RecastNavMeshGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavMesh/RecastNavMeshGenerator.h
---

# Navigation Bake Recast Detour Native Current-Source Algorithm Performance Review

## 1. Coverage and execution truth

The production Rust scope is **28/28 files**, **4,620 physical / 4,312 non-empty lines**, **149,474 bytes** and **1 inline test**. It covers 12 runtime bake files and 16 native/build wrapper files. The captured worktree is based on revision `39fe594bdaef6555277386dcc38362a575ada1c6`; its ordered fingerprint is `5b9589366caf67ee8bc4983846ba837634b23850a66a99dd9a5853b062f00563`.

The five owned C++ bridge files were also reviewed: `detour_query.cpp` 886 lines, `detour_crowd.cpp` 436, `detour_tile_cache.cpp` 796, `detour_tile_cache_raster.cpp` 188 and `recast_bake.cpp` 488. They are supporting evidence and are not counted in the Rust ledger.

## 2. Structural performance and correctness findings

### P0: bake input is placeholder geometry, so optimization data would be invalid

`manager/bake/geometry.rs:125` scans the World, but every render mesh/cube becomes a transformed 1x1 top quad (`:177-185`). Physics boxes become top quads; spheres/capsules/cylinders become discs; convex hull becomes an AABB top surface; triangle meshes and heightfields are skipped (`:207-256`). Area volumes classify the node origin rather than triangle overlap, and carve obstacles remove an entire source node on a coarse intersection.

Empty source may still create a synthetic surface and return a successful asset. Tests explicitly normalize these shortcuts: a Cube contributes exactly two triangles and carving one overlapping obstacle removes the whole source. Until real mesh/collider extraction and spatial classification exist, bake time, polygon count and path quality cannot be compared with another engine.

### P0: tiled and dirty bake repeatedly process whole geometry

`native/src/bake.rs:241-253` determines occupied tiles by scanning all triangles for every grid cell. Each selected tile then receives the full shared mesh buffers; native Recast copies all indices/areas and rasterizes the entire triangle set (`recast_bake.cpp:322-422`), relying on bounds clipping inside Recast. Complexity is approximately O(tiles * triangles) before voxel work.

Dirty rebuild still calls full `prepare_bake` (`manager/bake/dirty.rs:249`), reconstructs a complete tiled plan, clones prior plan/asset state, extracts every preserved tile into new assets and merges all rebuilt/preserved assets. `merge_tiled_assets` deduplicates vertices by rounded floating-point coordinates in a global HashMap (`native/src/bake.rs:320-365`). This is full geometry and asset reconstruction with a dirty-tile-shaped result, not incremental generation.

### P0: private task fan-out has no scheduler lifecycle contract

Async bake consumes an owned clone of the complete World, prepares it on a private `TaskPool`, clones the plan for every tile and spawns one task per tile (`manager/bake/task_pool.rs:77-203`). Dirty bake follows the same pattern. Settings/newer-bake changes remove handles from maps, but workers are not cancelled or joined; tests call this “retire” while detached work can continue consuming CPU and memory.

There is no bounded queue, dependency handle, priority, memory/work budget, deadline, cooperative cancellation or shutdown receipt. Harvest owns another full World plus preparation and result arrays, then clones asset/report/snapshot data during publication. Synchronous `bake_surface` performs all preparation and native work on the caller.

### P0: TileCache is a single lossy raster layer, not tiled navigation data

`detour_tile_cache_raster.cpp:138-147` clamps the whole asset to at most 160 cells per axis. For each cell, `area_at_cell` scans all polygons and triangles (`:43-70`, `:150-182`), O(cells * polygons * triangles-per-polygon). Heights are always zero. `detour_tile_cache.cpp:482-497` emits only tile `(0,0,0)`, then configures `maxTiles=4` and builds one compressed layer (`:531-579`). This discards vertical layering and does not preserve the baked tile topology.

Obstacle synchronization may run up to 64 zero-delta update iterations on the caller (`detour_tile_cache.cpp:343-355`). The implementation is only suitable for tiny flat fixtures, not an engine world.

### P1: fixed native limits and area flag folding are not product contracts

Path corridors and straight paths are fixed arrays of 512 in both normal Detour and TileCache query code. A result whose corridor does not end at the destination is reported as no path; partial/truncation semantics are discarded. Area IDs 16-63 all fold into flag bit 15 (`detour_query.cpp:43-48`) even though the filter exposes a 64-bit area mask. Query extents span roughly half the entire asset, increasing nearest-poly search work.

Advanced surface/settings fields are partially ignored or only hashed/warned. The dist/native capability therefore overstates the fidelity of Recast/Detour behavior.

## 3. Unreal source constraints

- `NavigationDirtyAreasController.cpp:55-116` accumulates dirty bounds and rebuilds at a configured frequency instead of rebuilding on every change/frame.
- `NavigationSystem.cpp:1661-1792` processes pending octree updates, dirty-area rebuild and async/time-sliced generators as separate instrumented phases; `:1796-1811` publishes remaining/running task counters.
- `RecastNavMeshGenerator.cpp:1832-1932` scopes geometry and cached compressed layers to a tile. `:6853-6862` turns dirty areas into dirty tiles rather than rerunning whole-world bake preparation.
- `RecastNavMeshGenerator.cpp:6738-6750` bounds task submission by available generator slots. `:8241-8345` owns pending/running tile queues, starts background tasks and applies only completed tile data. `:6704-6711` provides cancellation, while `:424-489` in the header defines time-sliced tile-generation state.

The transferable design is indexed geometry -> accumulated dirty bounds -> affected tile/layer tasks -> bounded async/time-sliced execution -> generation-checked commit. Zircon should not copy Unreal-specific allocation or object APIs.

## 4. Dependency-ordered optimization plan

### M0: define truth before performance

Reject or mark unsupported every geometry/settings path not implemented. Remove synthetic success from product bake. Record source kind, admitted triangle count, rejected source count, bounds, settings and provider version in the bake receipt.

### M1: create a navigation geometry index

At scene/resource admission, extract real render meshes and collider triangles/heightfields into immutable world-generation geometry chunks keyed by owner and bounds. Keep modifiers, obstacles and links in spatial indices. Parse components once and track change generations.

### M2: compile genuine tiled/layered Recast data

Partition source references spatially before workers. Each tile task receives only overlapping chunks plus border data and outputs native Detour tile/layer blobs with stable coordinates. Preserve vertical layers and exact area/link metadata. Eliminate polygon-pair adjacency reconstruction and rounded global vertex merging from the query path.

### M3: implement accumulated dirty bounds

Coalesce changes at a configurable frequency, map them through the geometry index to tile/layer coordinates and rebuild only affected tiles. Reuse unchanged compressed layers and native tile data by generation. A dirty update must not recollect or clone the full World, full geometry or preserved assets.

### M4: move work to the shared scheduler

Represent prepare, tile build and commit as dependency-tracked scheduler jobs with priority, concurrency, memory/time budgets, cooperative cancellation and shutdown receipts. The owner thread performs short extraction/commit only. Superseded settings/world generations cancel and join work rather than hiding handles.

### M5: publish transactional assets

Commit generated navigation through one generation-qualified transaction: native installed data, serialized output, asset/resource generation, diagnostics and undo snapshot either advance together or not at all. Store/reuse immutable blobs instead of cloning entire `NavMeshAsset` values through report/snapshot/task state.

### M6: dynamic qualification

Benchmark real mesh/collider scenes at `10k/100k/1m/10m` triangles, `1/100/10k` dirty tiles, vertical layers, areas/links and obstacle churn. Report extract, spatial query, raster, region, contour, mesh, compression, queue, wait and commit p50/p95/p99; bytes/tasks/cancel latency; CPU, wakeups, RSS and power. Compare full versus dirty work by actual input triangles and native tile bytes.

## 5. Acceptance gates

1. Real render mesh and collider geometry is admitted; unsupported input fails explicitly and no synthetic success is published.
2. Tile tasks receive only spatially overlapping geometry, with measured near-linear total input growth.
3. Dirty changes do not clone/rescan the World, full geometry, full plan or preserved assets.
4. Tile/layer identity, native data and serialized asset share one generation/currentness transaction.
5. Scheduler queue, cancellation, memory/time budgets and shutdown are observable and tested.
6. No fixed 512 path truncation or area-flag folding is silently reported as ordinary no-path behavior.
7. WPR/ETW and power results from a current-source executable meet declared budgets before acceptance.

## 6. Validation status

- Per-production-Rust-file static review: **28/28 complete**.
- Owned native C++ bridge review: **5/5 complete** for algorithm evidence.
- Cargo/native tests: **pending** because the managed Windows validation session is unavailable.
- Existing tests use tiny flat synthetic fixtures; ignored microbenchmarks do not qualify world-scale bake/query behavior.
- WPR/ETW: **pending**, no launchable current-source executable.
- RenderDoc: **not a bake/query profiler**; defer until generated nav debug rendering has a real current-source GPU path.
- Protected ledgers, commit and WeCom completion remain pending.
