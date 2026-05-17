# Navigation Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the remaining deep navigation gaps for persistent TileCache, native off-mesh links, imported model bake geometry, DetourCrowd agents, and tiled-world scheduling.

**Architecture:** Shared contracts and persisted `.znavmesh` DTOs stay in `zircon_runtime`; concrete Recast/Detour/TileCache/Crowd state stays private to `zircon_plugins/navigation/native`; runtime behavior stays in `zircon_plugins/navigation/runtime` behind `NavigationManager`. The work proceeds bottom-up from DTO/asset foundations to native owners, then runtime orchestration, then acceptance documentation.

**Tech Stack:** Rust, Cargo, serde/bincode DTOs, Recast/Detour/DetourTileCache/DetourCrowd C++ through `cc`, WSL `g++ 11.4.0` native smoke validation, focused plugin workspace Cargo tests.

---

## Execution Policy

- Stay on the existing `main` checkout. Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- Do not revert unrelated dirty worktree changes.
- Do not patch renderer compile blockers from this navigation lane unless a navigation change directly owns the failing code.
- Use `E:\cargo-targets\zircon-navigation-validation` for focused navigation Cargo validation.
- Run milestone testing stages at milestone boundaries, not after every small implementation slice.

## Source Map

- Modify `zircon_runtime/src/asset/assets/navigation.rs`: persisted tiled navmesh DTOs, cache-layer metadata, tile dirty status, source geometry fingerprints, serialization tests.
- Modify `zircon_runtime/src/core/framework/navigation/mod.rs`: runtime stats, tick report counters, and model-geometry bake cache APIs on `NavigationManager`.
- Update `docs/zircon_runtime/core/framework/navigation.md`: shared navigation contract docs.
- Create `docs/zircon_runtime/asset/assets/navigation.md`: source-path mirror for `.znavmesh` asset persistence when tile/cache metadata lands.
- Modify `zircon_plugins/navigation/native/build.rs`: compile `native/detour_crowd.cpp`.
- Modify `zircon_plugins/navigation/native/native/recast_bridge.h`: C ABI structs and function declarations for persistent TileCache, off-mesh cost data, and Crowd.
- Modify `zircon_plugins/navigation/native/native/detour_query.cpp`: off-mesh construction and cost behavior shared by static queries.
- Modify `zircon_plugins/navigation/native/native/detour_tile_cache.cpp`: persistent TileCache world owner, obstacle refs, dirty update budget, path query against mutable navmesh.
- Create `zircon_plugins/navigation/native/native/detour_crowd.cpp`: native DetourCrowd opaque owner and agent sync/update/readback.
- Modify `zircon_plugins/navigation/native/src/ffi.rs`: Rust C layout records and extern declarations.
- Modify `zircon_plugins/navigation/native/src/tile_cache.rs`: Rust RAII wrapper for persistent TileCache world and obstacle refs.
- Create `zircon_plugins/navigation/native/src/crowd.rs`: Rust RAII wrapper for native Crowd world and agent readback records.
- Modify `zircon_plugins/navigation/native/src/detour.rs`: shared off-mesh and static navmesh query wrapper changes for native link-cost support.
- Modify `zircon_plugins/navigation/native/src/lib.rs`: facade methods for persistent TileCache, off-mesh-native query behavior, and Crowd world creation.
- Modify `zircon_plugins/navigation/native/tests/detour_query.rs`: native Rust regressions for off-mesh costs, TileCache persistence, and carved/off-mesh combinations.
- Modify `zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp`: persistent TileCache native smoke harness.
- Create `zircon_plugins/navigation/native/tests/crowd_smoke.cpp`: direct native Crowd smoke harness.
- Modify `zircon_plugins/navigation/runtime/src/lib.rs`: private module wiring for new runtime modules and tests.
- Modify `zircon_plugins/navigation/runtime/src/manager.rs`: reduce to orchestration and `NavigationManager` implementation after extracting responsibilities.
- Create `zircon_plugins/navigation/runtime/src/loaded_navmesh.rs`: handle allocation, loaded records, selected asset/native state access.
- Create `zircon_plugins/navigation/runtime/src/bake_geometry.rs`: render/collider/model bake geometry collection and diagnostics.
- Create `zircon_plugins/navigation/runtime/src/tile_runtime.rs`: obstacle snapshots, TileCache runtime state, active/dirty tile queues, rebuild budgets.
- Create `zircon_plugins/navigation/runtime/src/crowd_runtime.rs`: `NavMeshAgent` sync, native crowd lifecycle, target requests, transform writeback helpers.
- Modify `zircon_plugins/navigation/runtime/src/runtime_obstacles.rs`: stable obstacle ids, movement thresholds, stationary timers, TileCache conversion records.
- Modify `zircon_runtime/src/asset/assets/model.rs`: add narrow primitive triangle-count and valid-triangle helpers used by navigation bake collection.
- Update `docs/zircon_plugins/navigation/native.md`: native ABI, TileCache, off-mesh, Crowd, and validation evidence.
- Update `docs/zircon_plugins/navigation/runtime.md`: loaded navmesh runtime state, bake geometry, TileCache scheduling, Crowd behavior, and stats.
- Update `tests/acceptance/navigation-tile-cache-carving.md`: transition from transient to persistent TileCache evidence.
- Create `tests/acceptance/navigation-persistent-tile-cache.md`: persistent TileCache acceptance record.
- Create `tests/acceptance/navigation-detour-crowd.md`: Crowd acceptance record.
- Create `tests/acceptance/navigation-model-geometry-bake.md`: imported model geometry bake acceptance record.
- Create `tests/acceptance/navigation-tiled-world-scheduling.md`: tiled-world dirty/active rebuild acceptance record.
- Create `tests/acceptance/navigation-gap-closure.md`: final residual-gap report and validation summary.

## Milestone 0: Baseline And Coordination

**Goal:** Confirm the active workspace state, known blockers, and validation baseline before touching production code.

**In-Scope Behaviors**

- Active session note reflects that implementation planning has started.
- Branch is `main`.
- Recent `.codex/sessions` and `.codex/plans` overlap has been checked.
- Current navigation Cargo blocker is recorded if it still stops before navigation tests.
- Target-drive free space is checked before heavy Cargo validation.

**Dependencies**

- Approved design spec: `docs/superpowers/specs/2026-05-04-navigation-gap-closure-design.md`.
- Current session note: `.codex/sessions/20260504-0324-navigation-gap-closure-design.md`.

**Implementation Slices**

- [ ] Update `.codex/sessions/20260504-0324-navigation-gap-closure-design.md` to `status: active-implementation-plan-ready` after this plan is accepted.
- [ ] Record current target directory policy in the session note: `E:\cargo-targets\zircon-navigation-validation`.
- [ ] Do not edit renderer files if focused Cargo stops in `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`.

**Testing Stage: Baseline Evidence**

- [ ] Run coordination scan:

```powershell
.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
```

Expected: lists active sessions and no unreviewed navigation-overlap plan that changes this milestone order.

- [ ] Confirm branch:

```powershell
git branch --show-current
```

Expected: `main`.

- [ ] Check target drive free space:

```powershell
Get-PSDrive -Name E
```

Expected: free space above `50 GB`; if `<= 50 GB`, clean `E:\cargo-targets\zircon-navigation-validation` before Cargo validation.

- [ ] Attempt a narrow blocker confirmation only if useful before implementation:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast tile_cache_carved_obstacle_blocks_corridor_path --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --exact --nocapture
```

Expected when renderer drift is still present: Cargo stops before navigation test execution with the unrelated renderer diagnostic. Record the exact diagnostic and continue with native/static evidence until that lane clears.

**Exit Evidence**

- Session note updated.
- Branch and coordination facts recorded.
- Validation blocker status is known and not guessed.

## Milestone 1: Tiled Asset And Framework DTO Foundation

**Goal:** Make tiled/cache/crowd metadata representable in neutral serialized DTOs before native or runtime behavior depends on it.

**In-Scope Behaviors**

- `NavMeshAsset` can persist tile coordinates, tile polygon/index ranges, build metadata, optional native payload bytes, optional compressed TileCache layer bytes, source geometry fingerprints, and dirty status.
- Existing `NavMeshAsset::simple_quad`, `from_triangle_mesh`, `to_bytes`, and `from_bytes` continue to work.
- Old serialized assets can deserialize through `serde(default)` for new fields.
- `NavigationRuntimeStats` and `NavAgentTickReport` expose tile, TileCache, and Crowd counters without native handles.

**Dependencies**

- Milestone 0 baseline is recorded.

**Implementation Slices**

- [ ] Modify `zircon_runtime/src/asset/assets/navigation.rs` and add neutral persisted records near the current asset structs:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshTileCoordAsset {
    pub x: i32,
    pub y: i32,
    pub layer: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavMeshTileBuildAsset {
    pub cell_size: Real,
    pub cell_height: Real,
    pub walkable_height: Real,
    pub walkable_radius: Real,
    pub walkable_climb: Real,
    pub max_slope_degrees: Real,
    pub border_size: u32,
    pub tile_size: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshTileCacheLayerAsset {
    pub coord: NavMeshTileCoordAsset,
    pub bounds_min: [i32; 3],
    pub bounds_max: [i32; 3],
    pub width: u16,
    pub height: u16,
    pub compressed: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshTileDirtyStateAsset {
    #[default]
    Clean,
    Dirty,
    RebuildPending,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavMeshSourceGeometryAsset {
    pub entity: Option<u64>,
    pub source_uri: Option<String>,
    pub primitive_index: Option<u32>,
    pub transform_hash: u64,
    pub geometry_hash: u64,
}
```

- [ ] Extend `NavMeshAsset` with defaulted fields:

```rust
#[serde(default)]
pub tile_build: Option<NavMeshTileBuildAsset>,
#[serde(default)]
pub tile_cache_layers: Vec<NavMeshTileCacheLayerAsset>,
#[serde(default)]
pub source_geometry: Vec<NavMeshSourceGeometryAsset>,
```

- [ ] Extend `NavMeshTileAsset` with defaulted fields:

```rust
#[serde(default)]
pub coord: NavMeshTileCoordAsset,
#[serde(default)]
pub first_polygon: u32,
#[serde(default)]
pub first_index: u32,
#[serde(default)]
pub index_count: u32,
#[serde(default)]
pub detour_payload: Vec<u8>,
#[serde(default)]
pub dirty_state: NavMeshTileDirtyStateAsset,
#[serde(default)]
pub last_error: Option<String>,
```

- [ ] Add `Default` to `NavMeshTileAsset` so tests and future tile helper constructors can use struct update syntax with new defaulted fields:

```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavMeshTileAsset {
    pub id: u32,
    pub bounds_min: [Real; 3],
    pub bounds_max: [Real; 3],
    pub polygon_count: u32,
    #[serde(default)]
    pub coord: NavMeshTileCoordAsset,
    #[serde(default)]
    pub first_polygon: u32,
    #[serde(default)]
    pub first_index: u32,
    #[serde(default)]
    pub index_count: u32,
    #[serde(default)]
    pub detour_payload: Vec<u8>,
    #[serde(default)]
    pub dirty_state: NavMeshTileDirtyStateAsset,
    #[serde(default)]
    pub last_error: Option<String>,
}
```

- [ ] Update `NavMeshAsset::empty`, `simple_quad`, and `from_triangle_mesh_with_areas` initializers so every new field has deterministic default values.
- [ ] In `simple_quad`, set tile ranges to `first_polygon: 0`, `first_index: 0`, and `index_count: 6`.
- [ ] In `from_triangle_mesh_with_areas`, set `first_polygon: 0`, `first_index: 0`, and `index_count: valid_indices.len() as u32` for the single produced tile.
- [ ] Add inline tests in `navigation.rs` for binary roundtrip preserving new tile fields:

```rust
#[test]
fn nav_mesh_binary_roundtrip_preserves_tile_metadata() {
    let mut asset = NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 2.0);
    asset.tile_build = Some(NavMeshTileBuildAsset {
        cell_size: 0.3,
        cell_height: 0.2,
        walkable_height: 2.0,
        walkable_radius: 0.5,
        walkable_climb: 0.4,
        max_slope_degrees: 45.0,
        border_size: 1,
        tile_size: 32,
    });
    asset.tiles[0].coord = NavMeshTileCoordAsset { x: 3, y: -2, layer: 1 };
    asset.tiles[0].dirty_state = NavMeshTileDirtyStateAsset::Dirty;
    asset.tile_cache_layers.push(NavMeshTileCacheLayerAsset {
        coord: asset.tiles[0].coord,
        bounds_min: [0, 0, 0],
        bounds_max: [4, 2, 4],
        width: 4,
        height: 4,
        compressed: vec![1, 2, 3, 4],
    });

    let decoded = NavMeshAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap();

    assert_eq!(decoded.tile_build, asset.tile_build);
    assert_eq!(decoded.tiles[0].coord, asset.tiles[0].coord);
    assert_eq!(decoded.tiles[0].dirty_state, NavMeshTileDirtyStateAsset::Dirty);
    assert_eq!(decoded.tile_cache_layers, asset.tile_cache_layers);
}
```

- [ ] Modify `zircon_runtime/src/core/framework/navigation/mod.rs` and extend `NavAgentTickReport`:

```rust
pub crowd_updated_agents: usize,
pub off_mesh_link_traversals: usize,
pub rebuilt_tiles: usize,
```

- [ ] Extend `NavigationRuntimeStats`:

```rust
pub loaded_tiles: usize,
pub active_tiles: usize,
pub pending_dirty_tiles: usize,
pub active_tile_cache_obstacles: usize,
pub active_crowd_agents: usize,
```

- [ ] Add framework tests that default reports and stats keep old zero behavior:

```rust
#[test]
fn runtime_stats_default_has_no_active_tiles_or_crowd_agents() {
    let stats = NavigationRuntimeStats::default();
    assert_eq!(stats.loaded_tiles, 0);
    assert_eq!(stats.active_tiles, 0);
    assert_eq!(stats.pending_dirty_tiles, 0);
    assert_eq!(stats.active_tile_cache_obstacles, 0);
    assert_eq!(stats.active_crowd_agents, 0);
}

#[test]
fn agent_tick_report_default_has_no_crowd_or_tile_activity() {
    let report = NavAgentTickReport::default();
    assert_eq!(report.crowd_updated_agents, 0);
    assert_eq!(report.off_mesh_link_traversals, 0);
    assert_eq!(report.rebuilt_tiles, 0);
}
```

- [ ] Create `docs/zircon_runtime/asset/assets/navigation.md` with the required YAML frontmatter and a detailed explanation of `.znavmesh` tiled persistence.
- [ ] Update `docs/zircon_runtime/core/framework/navigation.md` frontmatter and body for new stats/report fields and asset metadata.

**Testing Stage: DTO Foundation Validation**

- [ ] Run focused runtime navigation/framework tests:

```powershell
cargo test -p zircon_runtime nav_mesh_binary_roundtrip_preserves_tile_metadata runtime_stats_default_has_no_active_tiles_or_crowd_agents agent_tick_report_default_has_no_crowd_or_tile_activity --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never
```

Expected: tests pass if Cargo reaches `zircon_runtime`; if unrelated renderer compile drift stops compilation first, record the exact diagnostic.

- [ ] Run rustfmt check on touched runtime files:

```powershell
rustfmt --edition 2021 --check "zircon_runtime\src\asset\assets\navigation.rs" "zircon_runtime\src\core\framework\navigation\mod.rs"
```

- [ ] Run whitespace check:

```powershell
git diff --check -- "zircon_runtime\src\asset\assets\navigation.rs" "zircon_runtime\src\core\framework\navigation\mod.rs" "docs\zircon_runtime\asset\assets\navigation.md" "docs\zircon_runtime\core\framework\navigation.md" "docs\superpowers\plans\2026-05-04-navigation-gap-closure.md" "docs\superpowers\specs\2026-05-04-navigation-gap-closure-design.md"
```

**Exit Evidence**

- New DTOs serialize deterministically.
- Default stats/report behavior remains zeroed.
- Docs identify new persisted fields and their native-handle boundary.

## Milestone 2: Runtime Manager Modularization And Loaded Navmesh Records

**Goal:** Split `manager.rs` before adding more responsibilities and introduce loaded navmesh records that can own persistent native runtime state in later milestones.

**In-Scope Behaviors**

- `manager.rs` remains the `NavigationManager` implementation and orchestration boundary.
- Existing bake/query/tick behavior remains unchanged after extraction.
- Loaded navmesh handle allocation and selection move into `loaded_navmesh.rs`.
- Bake geometry collection moves into `bake_geometry.rs` without model-asset behavior yet.
- Runtime stats count loaded tiles through loaded navmesh records.

**Dependencies**

- Milestone 1 DTOs and stats fields exist.

**Implementation Slices**

- [ ] Create `zircon_plugins/navigation/runtime/src/loaded_navmesh.rs` with these public crate-local records:

```rust
use std::collections::HashMap;

use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{NavMeshHandle, NavigationError};

#[derive(Debug)]
pub(crate) struct LoadedNavMeshRecord {
    pub asset: NavMeshAsset,
}

#[derive(Debug)]
pub(crate) struct LoadedNavMeshes {
    next_handle: u64,
    loaded: HashMap<NavMeshHandle, LoadedNavMeshRecord>,
}

impl Default for LoadedNavMeshes {
    fn default() -> Self {
        Self { next_handle: 1, loaded: HashMap::new() }
    }
}
```

- [ ] Implement `insert`, `selected_asset`, `len`, and `loaded_tile_count` in `loaded_navmesh.rs`. `selected_asset` should keep the current lowest-handle fallback and clone the asset for existing backend methods.
- [ ] Create `zircon_plugins/navigation/runtime/src/bake_geometry.rs` and move `BakeGeometry`, `collect_bake_geometry`, render/collider collection, modifier lookup helpers, and bake diagnostics from `manager.rs` into it.
- [ ] Keep the moved API narrow:

```rust
pub(crate) fn collect_bake_geometry(
    world: &World,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
    agent_type: &str,
) -> BakeGeometry;

pub(crate) fn bake_geometry_diagnostics(
    geometry: &BakeGeometry,
    surface_entity: Option<u64>,
) -> Vec<NavMeshBakeDiagnostic>;
```

- [ ] Modify `zircon_plugins/navigation/runtime/src/lib.rs` to add private modules:

```rust
mod bake_geometry;
mod loaded_navmesh;
```

- [ ] Modify `manager.rs` so `NavigationRuntimeState` contains `loaded: LoadedNavMeshes` instead of `HashMap<NavMeshHandle, NavMeshAsset>`.
- [ ] Preserve existing public behavior for `load_nav_mesh`, `find_path`, `sample_position`, `raycast`, and `tick_world_agents`.
- [ ] Update `stats.loaded_nav_meshes` and `stats.loaded_tiles` from `LoadedNavMeshes`.
- [ ] Move existing related tests only if they follow the moved helpers; keep test names unchanged.
- [ ] Update `docs/zircon_plugins/navigation/runtime.md` to describe the manager split.

**Testing Stage: Modularization Validation**

- [ ] Run runtime plugin focused tests if Cargo reaches navigation:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never
```

Expected: existing runtime navigation tests pass or the unrelated renderer compile blocker is recorded before navigation tests execute.

- [ ] Run rustfmt check:

```powershell
rustfmt --edition 2021 --check "zircon_plugins\navigation\runtime\src\manager.rs" "zircon_plugins\navigation\runtime\src\loaded_navmesh.rs" "zircon_plugins\navigation\runtime\src\bake_geometry.rs" "zircon_plugins\navigation\runtime\src\lib.rs"
```

- [ ] Run whitespace check:

```powershell
git diff --check -- "zircon_plugins\navigation\runtime" "docs\zircon_plugins\navigation\runtime.md"
```

**Exit Evidence**

- `manager.rs` is smaller and orchestration-focused.
- Existing behavior is preserved before native persistence work starts.
- Loaded records are ready to receive TileCache and Crowd owners in later milestones.

## Milestone 3: Persistent TileCache Native And Runtime State

**Goal:** Replace transient per-query TileCache construction with a persistent owner tied to loaded navmesh records and stable obstacle refs.

**In-Scope Behaviors**

- Native C ABI creates/frees a persistent TileCache world.
- Runtime can add, update, and remove box/capsule carving obstacles by stable ids.
- TileCache update runs with a bounded tile budget and reports changed tile ids.
- Path queries use the persistent mutable navmesh when obstacles are active.
- Existing transient `find_path_with_obstacles` remains only as a compatibility fallback inside the native crate during migration, not as the primary runtime path.

**Dependencies**

- Milestone 1 DTO fields exist.
- Milestone 2 loaded navmesh records exist.

**Implementation Slices**

- [ ] Extend `zircon_plugins/navigation/native/native/recast_bridge.h` with opaque handles and records:

```cpp
struct ZrNavDetourTileCacheWorld;

struct ZrNavTileCacheObstacleRef {
    unsigned int id;
};

struct ZrNavTileCacheUpdateResult {
    int status;
    unsigned int changed_tile_count;
    const unsigned int* changed_tiles;
    const char* message;
};
```

- [ ] Add C ABI declarations for `zr_nav_detour_tile_cache_world_create`, `zr_nav_detour_tile_cache_world_free`, `zr_nav_detour_tile_cache_world_add_obstacle`, `zr_nav_detour_tile_cache_world_update_obstacle`, `zr_nav_detour_tile_cache_world_remove_obstacle`, `zr_nav_detour_tile_cache_world_update`, `zr_nav_detour_tile_cache_world_find_path`, and free functions for result buffers.
- [ ] Modify `native/detour_tile_cache.cpp` so the persistent owner stores `dtNavMesh`, `dtTileCache`, allocator, compressor, mesh-process callback, obstacle refs, and scratch query objects.
- [ ] Preserve the existing `zr_nav_detour_tile_cache_query_create`/query/free functions until Rust callers are moved, then route them through the persistent owner internally or remove them in the same milestone if no caller remains.
- [ ] Modify `zircon_plugins/navigation/native/src/ffi.rs` with matching `repr(C)` records and extern declarations.
- [ ] Modify `zircon_plugins/navigation/native/src/tile_cache.rs` to add:

```rust
pub struct RecastTileCacheWorld {
    raw: NonNull<ffi::ZrNavDetourTileCacheWorld>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecastTileCacheObstacleId(pub u32);
```

- [ ] Implement `Drop` for `RecastTileCacheWorld` and methods `create`, `add_obstacle`, `update_obstacle`, `remove_obstacle`, `update`, and `find_path`.
- [ ] Modify `zircon_plugins/navigation/native/src/lib.rs` to expose crate-public construction methods for runtime loaded records without exposing C handles.
- [ ] Create `zircon_plugins/navigation/runtime/src/tile_runtime.rs` with obstacle snapshots:

```rust
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeObstacleSnapshot {
    pub entity: u64,
    pub center: Vec3,
    pub radius: Real,
    pub descriptor_hash: u64,
    pub stationary_seconds: Real,
}
```

- [ ] Implement TileCache runtime state that maps world entity id to native obstacle id and tracks dirty tile ids returned from native update.
- [ ] Honor `move_threshold`, `time_to_stationary`, and `carve_only_stationary` when deciding whether a runtime obstacle is added to TileCache.
- [ ] Add `tile_runtime` to `lib.rs`.
- [ ] Extend `LoadedNavMeshRecord` with an optional TileCache runtime state. Use `Option` so unsupported native creation reports diagnostics without panics.
- [ ] Modify `tick_world_agents` to synchronize carving obstacles before path queries and increment `report.rebuilt_tiles` and `stats.pending_dirty_tiles` from `tile_runtime`.
- [ ] Keep `RecastBackend::find_path_with_obstacles` as a fallback only when no persistent owner exists; report fallback in diagnostics when obstacles are present.
- [ ] Update `zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp` so it creates one world, adds an obstacle, calls update, queries, removes the obstacle, updates again, and verifies the corridor becomes passable again.
- [ ] Add Rust native tests in `detour_query.rs` for persistent obstacle add/remove using the Rust wrapper.
- [ ] Add runtime test proving a moved obstacle marks tiles dirty and later unblocks a path after removal.
- [ ] Update `docs/zircon_plugins/navigation/native.md`, `docs/zircon_plugins/navigation/runtime.md`, `tests/acceptance/navigation-tile-cache-carving.md`, and create `tests/acceptance/navigation-persistent-tile-cache.md`.

**Testing Stage: Persistent TileCache Validation**

- [ ] Run WSL native TileCache smoke:

```powershell
wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER -I. -Izircon_plugins/navigation/native/vendor/recastnavigation/Recast/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/Detour/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Include zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp zircon_plugins/navigation/native/native/detour_tile_cache.cpp zircon_plugins/navigation/native/native/detour_query.cpp zircon_plugins/navigation/native/vendor/recastnavigation/Recast/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Source/*.cpp -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke"
```

Expected: output shows obstacle-added no-path and obstacle-removed complete-path cases.

- [ ] Run native Rust tests if Cargo reaches navigation:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast tile_cache --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run runtime obstacle tests if Cargo reaches navigation:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime obstacle --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run rustfmt check on touched Rust files and `git diff --check --` on touched navigation/docs/acceptance files.

**Exit Evidence**

- Persistent native TileCache smoke passes at C ABI level.
- Rust/runtime tests pass when Cargo reaches navigation or record the exact external blocker.
- Docs no longer describe TileCache as only transient.

## Milestone 4: Native Off-Mesh Link Completion

**Goal:** Make representable off-mesh links, including `cost_override`, work through native Detour and TileCache paths instead of forcing Rust graph fallback.

**In-Scope Behaviors**

- Native navmesh creation embeds active off-mesh links with area, directionality, and traversal flags.
- `cost_override` affects route choice for representable native paths.
- TileCache queries retain off-mesh links when obstacles are active.
- Path result points keep `off_mesh_link` flags.
- Area mask rejection works for off-mesh link areas.

**Dependencies**

- Milestone 3 persistent TileCache owner exists.

**Implementation Slices**

- [ ] Extend native C ABI link records in `recast_bridge.h` and `ffi.rs` with `cost_override`, `has_cost_override`, `bidirectional`, `area`, and endpoint arrays.
- [ ] Modify `native/detour_query.cpp` to include off-mesh connection arrays in `dtNavMeshCreateParams` for representable links.
- [ ] Modify native custom query filter so off-mesh area cost and explicit link cost are applied consistently. If Detour cannot supply link identity inside the filter, compute final path cost after `findPath` and use it for route choice only in cases where multiple corridors are evaluated by the wrapper.
- [ ] Remove any Rust-side condition that skips native Detour solely because `NavMeshLinkAsset::cost_override` is `Some`.
- [ ] Modify `native/detour_tile_cache.cpp` so TileCache mesh processing receives the same off-mesh links as static Detour creation.
- [ ] Update `zircon_plugins/navigation/native/tests/detour_query.rs` imports for off-mesh tests:

```rust
use zircon_runtime::asset::{NavMeshAsset, NavMeshLinkAsset, NavMeshPolygonAsset, NavMeshTileAsset};
use zircon_runtime::core::framework::navigation::{
    NavLinkTraversalMode, NavPathQuery, NavPathStatus, NavRaycastQuery, NavSampleQuery, AREA_JUMP,
    AREA_WALKABLE, DEFAULT_AREA_MASK,
};
```

- [ ] Add a native route-choice regression that fails if a high-cost direct off-mesh shortcut is preferred over a walkable corridor:

```rust
#[test]
fn native_off_mesh_cost_override_changes_route_choice() {
    let backend = RecastBackend;
    let mut asset = u_corridor_asset();
    asset.off_mesh_links.push(NavMeshLinkAsset {
        start: [0.5, 0.0, 0.5],
        end: [2.5, 0.0, 0.5],
        width: 0.0,
        bidirectional: true,
        area: AREA_JUMP,
        cost_override: Some(100.0),
        traversal_mode: NavLinkTraversalMode::Automatic,
    });

    let result = backend
        .find_path(
            &asset,
            &NavPathQuery::new([0.5, 0.0, 0.5], [2.5, 0.0, 0.5]),
        )
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert!(
        result
            .points
            .iter()
            .all(|point| !point.flags.iter().any(|flag| flag == "off_mesh_link")),
        "expected corridor route because direct off-mesh link is expensive: {result:?}"
    );
}
```

- [ ] Add a native area-mask regression that proves the link area controls reachability:

```rust
#[test]
fn native_off_mesh_link_area_mask_blocks_link() {
    let backend = RecastBackend;
    let mut asset = two_island_asset();
    asset.off_mesh_links.push(NavMeshLinkAsset {
        start: [0.9, 0.0, 0.5],
        end: [3.1, 0.0, 0.5],
        width: 0.0,
        bidirectional: true,
        area: AREA_JUMP,
        cost_override: None,
        traversal_mode: NavLinkTraversalMode::Automatic,
    });
    let mut query = NavPathQuery::new([0.2, 0.0, 0.5], [3.8, 0.0, 0.5]);
    query.area_mask = DEFAULT_AREA_MASK & !(1_u64 << AREA_JUMP);

    let result = backend.find_path(&asset, &query).unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}
```

- [ ] Add these helper assets to `detour_query.rs` for the off-mesh regressions:

```rust
fn u_corridor_asset() -> NavMeshAsset {
    navmesh_from_polygons(
        vec![
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 0.0, 2.0]],
            [[1.0, 0.0, 1.0], [2.0, 0.0, 1.0], [2.0, 0.0, 2.0], [1.0, 0.0, 2.0]],
            [[2.0, 0.0, 1.0], [3.0, 0.0, 1.0], [3.0, 0.0, 2.0], [2.0, 0.0, 2.0]],
            [[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [3.0, 0.0, 1.0], [2.0, 0.0, 1.0]],
        ],
        [0.0, 0.0, 0.0],
        [3.0, 0.0, 2.0],
    )
}

fn two_island_asset() -> NavMeshAsset {
    navmesh_from_polygons(
        vec![
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            [[3.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 0.0, 1.0], [3.0, 0.0, 1.0]],
        ],
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 1.0],
    )
}

fn navmesh_from_polygons(
    polygons: Vec<[[f32; 3]; 4]>,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
) -> NavMeshAsset {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut polygon_assets = Vec::new();
    for polygon in polygons {
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&polygon);
        let first_index = indices.len() as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        polygon_assets.push(NavMeshPolygonAsset {
            first_index,
            index_count: 6,
            area: AREA_WALKABLE,
            tile: 0,
        });
    }
    let polygon_count = polygon_assets.len() as u32;
    NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type: "humanoid".to_string(),
        settings_hash: 0,
        area_costs: NavMeshAsset::empty("humanoid").area_costs,
        vertices,
        indices,
        polygons: polygon_assets,
        tiles: vec![NavMeshTileAsset {
            id: 0,
            bounds_min,
            bounds_max,
            polygon_count,
            ..NavMeshTileAsset::default()
        }],
        off_mesh_links: Vec::new(),
        ..NavMeshAsset::empty("humanoid")
    }
}
```

- [ ] Ensure the off-mesh helper assets use the `NavMeshTileAsset::default()` implementation added in Milestone 1.
- [ ] Add a TileCache plus off-mesh test where an obstacle blocks the normal corridor and the active off-mesh link remains usable.
- [ ] Update native/runtime docs and acceptance records with off-mesh behavior.

**Testing Stage: Off-Mesh Validation**

- [ ] Run native off-mesh tests:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast off_mesh --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run TileCache tests that include off-mesh links:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast tile_cache --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run rustfmt and whitespace checks for touched native Rust, C++ bridge headers, docs, and tests.

**Exit Evidence**

- `cost_override` no longer forces native path fallback for representable links.
- TileCache does not drop off-mesh links.
- Masked off-mesh areas return `NoPath`.

## Milestone 5: Imported Model Geometry Bake Input

**Goal:** Feed real imported model primitive triangles into Recast bake collection instead of using only cube/mesh footprint fallback.

**In-Scope Behaviors**

- Render-mesh bake collection reads `ModelAsset.primitives[*].vertices` and `indices` when the world node has a resolvable model/mesh asset reference.
- World transforms are applied to every imported vertex position.
- Bake diagnostics distinguish model triangles, collider triangles, footprint fallback triangles, and missing model payloads.
- Static bind-pose geometry is supported; skinned runtime deformation remains out of scope.

**Dependencies**

- Milestone 2 `bake_geometry.rs` extraction exists.

**Implementation Slices**

- [ ] Modify `zircon_runtime/src/core/framework/navigation/mod.rs` so `NavigationManager` can accept imported model payloads without depending on editor/importer state:

```rust
fn cache_model_asset(&self, asset: ModelAsset) -> Result<(), NavigationError>;
fn remove_cached_model_asset(&self, model: ResourceHandle<ModelMarker>) -> Result<bool, NavigationError>;
```

- [ ] Import `ModelAsset`, `ModelMarker`, and `ResourceHandle` in `navigation/mod.rs` from the existing runtime asset/resource modules.
- [ ] Add `cached_models: HashMap<ResourceHandle<ModelMarker>, ModelAsset>` to `NavigationRuntimeState`.
- [ ] In `cache_model_asset`, use `ResourceId::from_locator(&asset.uri)` to key the model handle so scene `MeshRenderer.model` handles created from project references resolve to the same stable id.
- [ ] In `remove_cached_model_asset`, remove by handle and return whether a model was present.
- [ ] Modify `zircon_runtime/src/asset/assets/model.rs` with a narrow helper:

```rust
impl ModelAsset {
    pub fn primitive_triangle_count(&self) -> usize {
        self.primitives
            .iter()
            .map(ModelPrimitiveAsset::valid_triangle_count)
            .sum()
    }
}

impl ModelPrimitiveAsset {
    pub fn valid_triangle_count(&self) -> usize {
        self.indices
            .chunks(3)
            .filter(|triangle| {
                triangle.len() == 3
                    && triangle
                        .iter()
                        .all(|index| (*index as usize) < self.vertices.len())
            })
            .count()
    }
}
```

- [ ] Add `BakeGeometry` counters in `bake_geometry.rs`:

```rust
pub(crate) model_triangles: usize,
pub(crate) collider_triangles: usize,
pub(crate) footprint_triangles: usize,
pub(crate) missing_model_payloads: usize,
```

- [ ] Add a method that appends transformed primitive triangles:

```rust
pub(crate) fn push_model_primitive(
    &mut self,
    matrix: Mat4,
    vertices: &[MeshVertex],
    indices: &[u32],
    area: u8,
) {
    let base = self.vertices.len() as u32;
    self.vertices.extend(vertices.iter().map(|vertex| {
        matrix
            .transform_point3(Vec3::from_array(vertex.position))
            .to_array()
    }));
    for triangle in indices.chunks(3).filter(|triangle| triangle.len() == 3) {
        if triangle.iter().all(|index| (*index as usize) < vertices.len()) {
            self.indices.extend_from_slice(&[
                base + triangle[0],
                base + triangle[1],
                base + triangle[2],
            ]);
            self.triangle_areas.push(area);
            self.model_triangles += 1;
        }
    }
}
```

- [ ] Update render-node geometry collection to try model primitive geometry first when model payload is available, then footprint fallback only when no usable primitive triangles exist.
- [ ] Change `collect_bake_geometry` signature to receive the cached model map:

```rust
pub(crate) fn collect_bake_geometry(
    world: &World,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
    agent_type: &str,
    cached_models: &HashMap<ResourceHandle<ModelMarker>, ModelAsset>,
) -> BakeGeometry;
```

- [ ] In `manager.rs`, clone or borrow the cached model map while holding the navigation state lock only long enough to prepare bake collection, then release the lock before native Recast bake work.
- [ ] Emit warning diagnostics for missing model payloads with the entity id.
- [ ] Update `NavMeshBakeReport` population so `source_vertices`, `source_triangles`, and diagnostics reflect model geometry counts.
- [ ] Add runtime test that constructs a world with a model primitive containing a non-square triangle shape and asserts bake source triangle count equals imported primitive triangle count.
- [ ] Add runtime test where a mesh node references an unavailable model payload and asserts a warning diagnostic includes that entity id.
- [ ] Update `docs/zircon_plugins/navigation/runtime.md` and create `tests/acceptance/navigation-model-geometry-bake.md`.

**Testing Stage: Model Geometry Validation**

- [ ] Run runtime bake tests:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime bake --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run focused runtime check if tests are blocked before navigation:

```powershell
cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never
```

- [ ] Run rustfmt and whitespace checks for touched runtime/model/docs files.

**Exit Evidence**

- At least one test proves imported primitive triangles drive bake input.
- Missing model payloads produce diagnostics instead of silent high-fidelity bake claims.

## Milestone 6: DetourCrowd Native Wrapper And Runtime Agents

**Goal:** Use DetourCrowd for loaded-navmesh agent simulation and keep conservative movement only as a reported fallback.

**In-Scope Behaviors**

- Native C ABI owns `dtCrowd` behind an opaque handle.
- Rust wrapper syncs agent descriptors, positions, velocities, targets, area masks, and avoidance quality.
- Runtime agent ticks use Crowd when a loaded navmesh supports it.
- `update_position` and `update_rotation` remain respected.
- Runtime stats and tick reports count active crowd agents and crowd-updated agents.

**Dependencies**

- Milestone 3 persistent loaded native state exists.
- Milestone 4 native off-mesh construction works with representable links.

**Implementation Slices**

- [ ] Create `zircon_plugins/navigation/native/native/detour_crowd.cpp` with an opaque `ZrNavDetourCrowdWorld` that stores `dtCrowd`, navmesh pointer or owned navmesh data as required by Detour lifetime rules, agent refs, and result buffers.
- [ ] Extend `recast_bridge.h` with C ABI records:

```cpp
struct ZrNavCrowdAgentParams {
    float radius;
    float height;
    float max_acceleration;
    float max_speed;
    float collision_query_range;
    float path_optimization_range;
    unsigned char update_flags;
    unsigned char obstacle_avoidance_type;
    unsigned char separation_weight;
    unsigned char priority;
    unsigned long long area_mask;
};

struct ZrNavCrowdAgentState {
    unsigned int agent_id;
    float position[3];
    float velocity[3];
    int active;
    int target_state;
    int traversing_off_mesh_link;
};
```

- [ ] Add C ABI functions for create/free, add/update/remove agent, request move target, update crowd, copy states, and free state buffers.
- [ ] Modify `build.rs` to compile `native/detour_crowd.cpp`.
- [ ] Create `zircon_plugins/navigation/native/src/crowd.rs` with RAII wrapper `RecastCrowdWorld` and crate-public `RecastCrowdAgentId`.
- [ ] Add `mod crowd;` in native `lib.rs` and facade methods to create a crowd owner from a loaded navmesh asset.
- [ ] Create `zircon_plugins/navigation/runtime/src/crowd_runtime.rs` with agent sync state keyed by entity id.
- [ ] Map `NavAvoidanceQuality` to Detour obstacle avoidance quality:

```rust
fn avoidance_quality_index(quality: NavAvoidanceQuality) -> u8 {
    match quality {
        NavAvoidanceQuality::None => 0,
        NavAvoidanceQuality::Low => 1,
        NavAvoidanceQuality::Medium => 2,
        NavAvoidanceQuality::High => 3,
    }
}
```

- [ ] In `tick_world_agents`, choose Crowd when a loaded record has a crowd owner; otherwise preserve current conservative movement and append a diagnostic only when a loaded navmesh expected Crowd but native creation failed.
- [ ] Apply Crowd readback to world transforms only when `agent.update_position` is true.
- [ ] Preserve rotation update with current direction/velocity when `agent.update_rotation` is true.
- [ ] Count a readback state with `traversing_off_mesh_link != 0` as one `report.off_mesh_link_traversals` increment for that tick.
- [ ] Add native smoke `crowd_smoke.cpp`: two agents request the same target and update for several steps without occupying the exact same XZ position.
- [ ] Add runtime tests for two agents, destination update, disabled `update_position`, no-loaded-navmesh fallback, and stats/report counters.
- [ ] Update docs and create `tests/acceptance/navigation-detour-crowd.md`.

**Testing Stage: Crowd Validation**

- [ ] Run WSL native Crowd smoke:

```powershell
wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER -I. -Izircon_plugins/navigation/native/vendor/recastnavigation/Recast/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/Detour/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Include zircon_plugins/navigation/native/tests/crowd_smoke.cpp zircon_plugins/navigation/native/native/detour_crowd.cpp zircon_plugins/navigation/native/native/detour_query.cpp zircon_plugins/navigation/native/vendor/recastnavigation/Recast/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Source/*.cpp -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_crowd_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_crowd_smoke"
```

Expected: output shows active agents with distinct final XZ positions and no native allocation failure.

- [ ] Run native Crowd Rust tests if Cargo reaches navigation:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast crowd --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run runtime agent tests:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime agent --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run rustfmt and whitespace checks for touched Crowd/native/runtime/docs files.

**Exit Evidence**

- Native Crowd smoke executes.
- Runtime reports Crowd-updated agents when a loaded navmesh supports native Crowd.
- Conservative movement remains available only as a reported fallback.

## Milestone 7: Large-World Tile Activation And Rebuild Scheduling

**Goal:** Make `override_tile_size` drive tiled bake/runtime behavior and add deterministic active/dirty tile scheduling.

**In-Scope Behaviors**

- Bake output can contain multiple tile records with coordinates and polygon/index ranges.
- Runtime chooses active tiles around agents with destinations.
- Dirty inactive tiles stay pending.
- Dirty active tiles rebuild through a bounded update budget.
- Queries across adjacent active tiles work.

**Dependencies**

- Milestone 1 tile DTOs exist.
- Milestone 3 TileCache dirty update path exists.
- Milestone 6 agents can serve as tile invokers.

**Implementation Slices**

- [ ] Extend native Recast bake input and Rust `RecastBakeMeshInput` with an optional tile size:

```rust
pub tile_size: Option<u32>,
```

- [ ] In runtime `bake_surface`, pass `surface.override_tile_size` into the native bake input.
- [ ] Modify native `recast_bake.cpp` to partition source bounds into tiles when tile size is present. Each output polygon must receive the correct tile id.
- [ ] Use XZ tile coordinates for `NavMeshTileCoordAsset`: `x = floor((tile_min_x - bounds_min_x) / tile_world_size)`, `y = floor((tile_min_z - bounds_min_z) / tile_world_size)`, and `layer = 0` until height-layer output is implemented.
- [ ] Populate `NavMeshTileAsset.coord`, `first_polygon`, `first_index`, `index_count`, and bounds for each produced tile.
- [ ] In `tile_runtime.rs`, define active tile selection records:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RuntimeTileCoord {
    pub x: i32,
    pub y: i32,
    pub layer: u16,
}
```

- [ ] Add active tile computation using agent position and conservative radius. The initial default radius should cover at least the current tile and its immediate neighbors.
- [ ] Add dirty queue state with deterministic ordering by tile id.
- [ ] Add a rebuild budget constant local to `tile_runtime.rs`:

```rust
const DEFAULT_TILE_REBUILD_BUDGET_PER_TICK: usize = 4;
```

- [ ] Update stats fields `active_tiles` and `pending_dirty_tiles` on every tick.
- [ ] Add runtime tests for multi-tile bake persistence, active tile selection around an agent, inactive dirty tile deferral, active dirty tile rebuild, and cross-tile query.
- [ ] Update docs and create `tests/acceptance/navigation-tiled-world-scheduling.md`.

**Testing Stage: Tiled-World Validation**

- [ ] Run native/runtime tiled tests:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast tile --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime tile --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never -- --nocapture
```

- [ ] Run rustfmt and whitespace checks for touched bake/runtime/docs files.

**Exit Evidence**

- Multi-tile assets persist tile metadata.
- Active/dirty queue behavior is deterministic.
- Tile rebuild budget appears in tick reports or runtime stats.

## Milestone 8: Final Navigation Acceptance And Residual Gap Report

**Goal:** Consolidate docs, run focused validation, and report only real residual gaps or external blockers.

**In-Scope Behaviors**

- Native and runtime docs are current.
- Acceptance records contain commands, observed output, and blocker status.
- Final report distinguishes closed gaps, validation blocked by unrelated renderer code, and future out-of-scope items.
- No broad workspace green claim is made unless broad validation actually passes.

**Dependencies**

- Milestones 1 through 7 are implemented or explicitly blocked with evidence.

**Implementation Slices**

- [ ] Update `docs/zircon_plugins/navigation/native.md` frontmatter and body with all native files, Crowd, persistent TileCache, off-mesh behavior, and test evidence.
- [ ] Update `docs/zircon_plugins/navigation/runtime.md` frontmatter and body with manager modules, loaded records, bake geometry, TileCache scheduling, Crowd, tiled-world stats, and known fallback rules.
- [ ] Update `docs/zircon_runtime/core/framework/navigation.md` and `docs/zircon_runtime/asset/assets/navigation.md` for final DTO shape.
- [ ] Ensure acceptance docs under `tests/acceptance/` include command text, observed results, and external blockers.
- [ ] Create `tests/acceptance/navigation-gap-closure.md` with sections `Closed Gaps`, `Validation Evidence`, `External Blockers`, and `Remaining Future Work`.
- [ ] Archive or delete `.codex/sessions/20260504-0324-navigation-gap-closure-design.md` only after the user receives final implementation results.

**Testing Stage: Final Focused Matrix**

- [ ] Run formatting checks:

```powershell
rustfmt --edition 2021 --check "zircon_runtime\src\asset\assets\navigation.rs" "zircon_runtime\src\core\framework\navigation\mod.rs" "zircon_plugins\navigation\native\src\lib.rs" "zircon_plugins\navigation\native\src\ffi.rs" "zircon_plugins\navigation\native\src\detour.rs" "zircon_plugins\navigation\native\src\tile_cache.rs" "zircon_plugins\navigation\native\src\crowd.rs" "zircon_plugins\navigation\runtime\src\lib.rs" "zircon_plugins\navigation\runtime\src\manager.rs" "zircon_plugins\navigation\runtime\src\loaded_navmesh.rs" "zircon_plugins\navigation\runtime\src\bake_geometry.rs" "zircon_plugins\navigation\runtime\src\tile_runtime.rs" "zircon_plugins\navigation\runtime\src\crowd_runtime.rs" "zircon_plugins\navigation\runtime\src\runtime_obstacles.rs"
```

- [ ] Run native plugin tests:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_recast --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never
```

- [ ] Run runtime plugin tests:

```powershell
cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-navigation-validation" --message-format short --color never
```

- [ ] Run WSL native smoke harnesses if Windows Cargo cannot reach native tests:

```powershell
wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "/mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_crowd_smoke"
```

- [ ] Run whitespace checks:

```powershell
git diff --check -- "zircon_runtime\src\asset\assets\navigation.rs" "zircon_runtime\src\core\framework\navigation\mod.rs" "zircon_plugins\navigation" "docs\zircon_plugins\navigation" "docs\zircon_runtime\core\framework\navigation.md" "docs\zircon_runtime\asset\assets\navigation.md" "tests\acceptance\navigation-tile-cache-carving.md" "tests\acceptance\navigation-persistent-tile-cache.md" "tests\acceptance\navigation-detour-crowd.md" "tests\acceptance\navigation-model-geometry-bake.md" "tests\acceptance\navigation-tiled-world-scheduling.md" "tests\acceptance\navigation-gap-closure.md" "docs\superpowers\specs\2026-05-04-navigation-gap-closure-design.md" "docs\superpowers\plans\2026-05-04-navigation-gap-closure.md"
```

- [ ] Optional broad validation only if focused navigation validation passes and unrelated lanes are stable:

```powershell
.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "E:\cargo-targets\zircon-navigation-validation"
```

Expected: if this command is not run or fails outside navigation, final reporting must not claim workspace-wide green.

**Exit Evidence**

- Focused navigation validation evidence is recorded.
- WSL native evidence covers native C++ when Cargo is externally blocked.
- Final residual-gap report is concrete and does not hide blockers.

## Plan Self-Review Results

- Spec coverage: M1 covers DTO/asset foundations; M3 covers persistent TileCache; M4 covers native off-mesh; M5 covers imported model geometry; M6 covers DetourCrowd; M7 covers tiled-world scheduling; M8 covers docs and final gap report.
- Text scan: no banned planning markers or unnamed validation steps remain in the executable plan body.
- Type consistency: DTO names use `NavMesh*Asset` for persisted data, `Recast*` for native Rust wrappers, and runtime modules remain crate-private under `zircon_plugins/navigation/runtime/src`.
- Boundary consistency: no native handle is placed in `zircon_runtime` shared DTOs, and `manager.rs` is split before new runtime behavior accumulates.
