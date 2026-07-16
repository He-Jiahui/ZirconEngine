use std::collections::{HashMap, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_plugin_navigation_recast::{merge_tiled_assets, RecastTileSpec, RecastTiledBakePlan};
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavMeshPolygonAsset, NavMeshTileAsset,
};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeReport, NavMeshBakeRequest, NavigationError, NavigationErrorKind,
    NavigationGeneratedBakeSnapshot,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::World;

use super::task_pool::NavMeshBakeTaskHandle;
use super::{
    bake_runtime_counts, canonical_surface_key, finish_bake, prepare_bake, tiled, BakePreparation,
};
use crate::manager::state::TiledBakeIdentity;
use crate::manager::DefaultNavigationManager;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavMeshDirtyBounds {
    pub min: [Real; 3],
    pub max: [Real; 3],
}

impl NavMeshDirtyBounds {
    pub fn new(min: [Real; 3], max: [Real; 3]) -> Self {
        Self { min, max }
    }

    fn validate(self) -> Result<Self, NavigationError> {
        if self
            .min
            .into_iter()
            .chain(self.max)
            .any(|coordinate| !coordinate.is_finite())
            || self.max[0] <= self.min[0]
            || self.max[1] <= self.min[1]
            || self.max[2] <= self.min[2]
        {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "navigation dirty bounds must be finite and non-empty",
            ));
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavMeshDirtyBakeReport {
    pub report: NavMeshBakeReport,
    pub rebuilt_tile_ids: Vec<u32>,
    pub preserved_tile_ids: Vec<u32>,
}

#[derive(Debug)]
enum PendingDirtyBakeState {
    Preparing,
    Baking {
        completed: usize,
        results: Vec<Option<Result<NavMeshAsset, NavigationError>>>,
        preparation: BakePreparation,
        world: World,
        plan: RecastTiledBakePlan,
        identity: TiledBakeIdentity,
        generation: u64,
        rebuilt_tile_ids: Vec<u32>,
        preserved_tile_ids: Vec<u32>,
    },
    Failed(NavigationError),
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct PendingDirtyBake {
    shared: Arc<Mutex<PendingDirtyBakeState>>,
    surface_entity: Option<u64>,
}

impl PendingDirtyBake {
    pub(super) fn is_ready(&self) -> bool {
        let state = lock_task(&self.shared);
        match &*state {
            PendingDirtyBakeState::Preparing => false,
            PendingDirtyBakeState::Baking {
                completed, results, ..
            } => *completed == results.len(),
            PendingDirtyBakeState::Failed(_) => true,
        }
    }

    pub(in crate::manager) fn surface_entity(&self) -> Option<u64> {
        self.surface_entity
    }
}

fn lock_task(shared: &Mutex<PendingDirtyBakeState>) -> MutexGuard<'_, PendingDirtyBakeState> {
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn panic_error(context: &str) -> NavigationError {
    NavigationError::new(
        NavigationErrorKind::BackendFailure,
        format!("navigation dirty {context} panicked"),
    )
}

impl DefaultNavigationManager {
    pub fn start_dirty_tile_rebuild(
        &self,
        world: World,
        request: NavMeshBakeRequest,
        dirty_bounds: NavMeshDirtyBounds,
    ) -> NavMeshBakeTaskHandle {
        let surface_entity = canonical_surface_key(&world, request.surface_entity);
        let shared = Arc::new(Mutex::new(PendingDirtyBakeState::Preparing));
        let (generation, handle) = {
            let mut state = self.lock_state();
            let generation = state.advance_bake_context(surface_entity);
            let handle = NavMeshBakeTaskHandle(state.next_bake_task);
            state.next_bake_task = state.next_bake_task.saturating_add(1);
            state.dirty_bake_tasks.insert(
                handle,
                PendingDirtyBake {
                    shared: Arc::clone(&shared),
                    surface_entity,
                },
            );
            (generation, handle)
        };
        let manager = self.clone();
        self.bake_pool.spawn(move || {
            let shared_for_work = Arc::clone(&shared);
            if catch_unwind(AssertUnwindSafe(|| {
                manager.prepare_dirty_task(
                    world,
                    request,
                    dirty_bounds,
                    surface_entity,
                    generation,
                    shared_for_work,
                );
            }))
            .is_err()
            {
                *lock_task(&shared) =
                    PendingDirtyBakeState::Failed(panic_error("tile preparation"));
            }
        });
        handle
    }

    pub fn try_harvest_dirty_tile_rebuild(
        &self,
        handle: NavMeshBakeTaskHandle,
    ) -> Option<Result<NavMeshDirtyBakeReport, NavigationError>> {
        let task = {
            let mut state = self.lock_state();
            if !state.dirty_bake_tasks.get(&handle)?.is_ready() {
                return None;
            }
            state.dirty_bake_tasks.remove(&handle)?
        };
        Some(self.finish_dirty_task(task))
    }

    fn prepare_dirty_task(
        &self,
        world: World,
        request: NavMeshBakeRequest,
        dirty_bounds: NavMeshDirtyBounds,
        surface_entity: Option<u64>,
        generation: u64,
        shared: Arc<Mutex<PendingDirtyBakeState>>,
    ) {
        let prepared = self.prepare_dirty_bake(&world, request, dirty_bounds, surface_entity);
        let (preparation, identity, plan, rebuilt_tile_ids, preserved_tile_ids, mut results) =
            match prepared {
                Ok(prepared) => prepared,
                Err(error) => {
                    *lock_task(&shared) = PendingDirtyBakeState::Failed(error);
                    return;
                }
            };
        let completed = results.iter().filter(|result| result.is_some()).count();
        let pending_tiles = plan
            .tiles()
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, _)| results[*index].is_none())
            .collect::<Vec<_>>();
        *lock_task(&shared) = PendingDirtyBakeState::Baking {
            completed,
            results: std::mem::take(&mut results),
            preparation,
            world,
            plan: plan.clone(),
            identity,
            generation,
            rebuilt_tile_ids: rebuilt_tile_ids.clone(),
            preserved_tile_ids,
        };

        for (index, tile) in pending_tiles {
            let shared = Arc::clone(&shared);
            let plan = plan.clone();
            let backend = self.backend.clone();
            self.bake_pool.spawn(move || {
                let result =
                    catch_unwind(AssertUnwindSafe(|| backend.bake_planned_tile(&plan, tile)))
                        .unwrap_or_else(|_| Err(panic_error("tile worker")));
                let mut state = lock_task(&shared);
                if let PendingDirtyBakeState::Baking {
                    completed, results, ..
                } = &mut *state
                {
                    results[index] = Some(result);
                    *completed += 1;
                }
            });
        }
    }

    fn prepare_dirty_bake(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
        dirty_bounds: NavMeshDirtyBounds,
        surface_entity: Option<u64>,
    ) -> Result<DirtyPrepared, NavigationError> {
        let dirty_bounds = dirty_bounds.validate()?;
        let previous = self
            .lock_state()
            .bake_contexts
            .get(&surface_entity)
            .and_then(|context| context.last_tiled_bake.clone())
            .ok_or_else(|| {
                NavigationError::new(
                    NavigationErrorKind::InvalidConfiguration,
                    "dirty tile rebuild requires a previously completed tiled bake",
                )
            })?;
        let preparation = prepare_bake(self, world, request)?;
        let identity = preparation.tiled_identity();
        if previous.identity != identity {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "dirty tile rebuild identity changed; request a full rebuild for the selected surface, agent, or settings",
            ));
        }
        let geometry_is_empty = preparation.geometry.source_triangles() == 0;
        let plan = match tiled::plan_for_preparation(self, &preparation)? {
            Some(plan) => plan,
            None if geometry_is_empty && preparation.surface.override_tile_size.is_some() => {
                previous.plan.clone()
            }
            None => {
                return Err(NavigationError::new(
                    NavigationErrorKind::InvalidConfiguration,
                    "dirty tile rebuild requires a tiled surface",
                ));
            }
        };
        ensure_compatible_tile_size(&previous.plan, &plan)?;
        let plan = plan.with_tiles(reconcile_tile_specs(&previous.plan, &plan)?);
        let expanded_min = [
            dirty_bounds.min[0] - plan.tile_size(),
            dirty_bounds.min[1],
            dirty_bounds.min[2] - plan.tile_size(),
        ];
        let expanded_max = [
            dirty_bounds.max[0] + plan.tile_size(),
            dirty_bounds.max[1],
            dirty_bounds.max[2] + plan.tile_size(),
        ];
        let rebuilt_tile_ids = plan
            .tiles()
            .iter()
            .filter(|tile| {
                overlaps_xz(tile.bounds_min, tile.bounds_max, expanded_min, expanded_max)
            })
            .map(|tile| tile.id)
            .collect::<Vec<_>>();
        let rebuilt_set = rebuilt_tile_ids.iter().copied().collect::<HashSet<_>>();
        let previous_set = previous
            .plan
            .tiles()
            .iter()
            .map(|tile| tile.id)
            .collect::<HashSet<_>>();
        if plan
            .tiles()
            .iter()
            .any(|tile| !rebuilt_set.contains(&tile.id) && !previous_set.contains(&tile.id))
        {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "dirty bounds do not cover newly occupied navigation tiles",
            ));
        }
        let preserved_tile_ids = previous
            .plan
            .tiles()
            .iter()
            .map(|tile| tile.id)
            .filter(|tile_id| !rebuilt_set.contains(tile_id))
            .collect::<Vec<_>>();
        let mut results = std::iter::repeat_with(|| None)
            .take(plan.tiles().len())
            .collect::<Vec<_>>();
        for (index, tile) in plan.tiles().iter().copied().enumerate() {
            if rebuilt_set.contains(&tile.id) {
                if geometry_is_empty {
                    results[index] = Some(Ok(empty_tile_asset(&preparation.agent_type, tile)));
                }
            } else {
                results[index] = Some(extract_tile_asset(&previous.asset, tile.id));
            }
        }
        Ok((
            preparation,
            identity,
            plan,
            rebuilt_tile_ids,
            preserved_tile_ids,
            results,
        ))
    }

    fn finish_dirty_task(
        &self,
        task: PendingDirtyBake,
    ) -> Result<NavMeshDirtyBakeReport, NavigationError> {
        let context_surface = task.surface_entity;
        let state = std::mem::replace(
            &mut *lock_task(&task.shared),
            PendingDirtyBakeState::Preparing,
        );
        let (
            mut results,
            preparation,
            world,
            plan,
            identity,
            generation,
            rebuilt_tile_ids,
            preserved_tile_ids,
        ) = match state {
            PendingDirtyBakeState::Baking {
                results,
                preparation,
                world,
                plan,
                identity,
                generation,
                rebuilt_tile_ids,
                preserved_tile_ids,
                ..
            } => (
                results,
                preparation,
                world,
                plan,
                identity,
                generation,
                rebuilt_tile_ids,
                preserved_tile_ids,
            ),
            PendingDirtyBakeState::Failed(error) => return Err(error),
            PendingDirtyBakeState::Preparing => {
                return Err(NavigationError::new(
                    NavigationErrorKind::BackendFailure,
                    "navigation dirty bake was harvested before completion",
                ));
            }
        };
        let mut assets = Vec::with_capacity(results.len());
        for result in &mut results {
            assets.push(
                result
                    .take()
                    .expect("completed dirty tile result missing")?,
            );
        }
        let mut asset = merge_tiled_assets(preparation.agent_type.clone(), assets)?;
        let report = finish_bake(&world, preparation, &mut asset);
        let generated_snapshot = NavigationGeneratedBakeSnapshot {
            surface_entity: context_surface,
            asset: report.asset.clone(),
            output_asset: report.output_asset.clone(),
        };
        self.publish_bake(
            context_surface,
            generation,
            Some((identity, plan, asset)),
            generated_snapshot,
            report.diagnostics.clone(),
            bake_runtime_counts(&world),
        )?;
        Ok(NavMeshDirtyBakeReport {
            report,
            rebuilt_tile_ids,
            preserved_tile_ids,
        })
    }
}

type DirtyPrepared = (
    BakePreparation,
    TiledBakeIdentity,
    RecastTiledBakePlan,
    Vec<u32>,
    Vec<u32>,
    Vec<Option<Result<NavMeshAsset, NavigationError>>>,
);

fn empty_tile_asset(agent_type: &str, tile: RecastTileSpec) -> NavMeshAsset {
    let mut asset = NavMeshAsset::empty(agent_type.to_string());
    asset.tiles.push(NavMeshTileAsset {
        id: tile.id,
        bounds_min: tile.bounds_min,
        bounds_max: tile.bounds_max,
        polygon_count: 0,
    });
    asset
}

fn ensure_compatible_tile_size(
    previous: &RecastTiledBakePlan,
    current: &RecastTiledBakePlan,
) -> Result<(), NavigationError> {
    if (previous.tile_size() - current.tile_size()).abs() <= Real::EPSILON {
        Ok(())
    } else {
        Err(NavigationError::new(
            NavigationErrorKind::InvalidConfiguration,
            "dirty geometry changed the navigation tile size; request a full rebuild",
        ))
    }
}

fn reconcile_tile_specs(
    previous: &RecastTiledBakePlan,
    current: &RecastTiledBakePlan,
) -> Result<Vec<RecastTileSpec>, NavigationError> {
    let mut previous_by_bounds = previous
        .tiles()
        .iter()
        .copied()
        .map(|tile| (tile_bounds_key(tile), tile))
        .collect::<HashMap<_, _>>();
    let mut next_id = previous
        .tiles()
        .iter()
        .map(|tile| tile.id)
        .max()
        .map_or(Some(0), |id| id.checked_add(1));
    let current_height = current
        .tiles()
        .first()
        .map(|tile| (tile.bounds_min[1], tile.bounds_max[1]));
    let mut tiles = Vec::with_capacity(previous.tiles().len() + current.tiles().len());

    for mut tile in current.tiles().iter().copied() {
        if let Some(previous_tile) = previous_by_bounds.remove(&tile_bounds_key(tile)) {
            tile.id = previous_tile.id;
            tile.x = previous_tile.x;
            tile.z = previous_tile.z;
        } else {
            let id = next_id.ok_or_else(|| {
                NavigationError::new(
                    NavigationErrorKind::InvalidConfiguration,
                    "navigation tile id space is exhausted",
                )
            })?;
            tile.id = id;
            next_id = id.checked_add(1);
        }
        tiles.push(tile);
    }
    for mut tile in previous_by_bounds.into_values() {
        if let Some((min_y, max_y)) = current_height {
            tile.bounds_min[1] = min_y;
            tile.bounds_max[1] = max_y;
        }
        tiles.push(tile);
    }
    tiles.sort_by_key(|tile| tile.id);
    Ok(tiles)
}

fn tile_bounds_key(tile: RecastTileSpec) -> [u32; 4] {
    [
        tile.bounds_min[0].to_bits(),
        tile.bounds_min[2].to_bits(),
        tile.bounds_max[0].to_bits(),
        tile.bounds_max[2].to_bits(),
    ]
}

fn overlaps_xz(
    left_min: [Real; 3],
    left_max: [Real; 3],
    right_min: [Real; 3],
    right_max: [Real; 3],
) -> bool {
    left_max[0] > right_min[0]
        && left_min[0] < right_max[0]
        && left_max[2] > right_min[2]
        && left_min[2] < right_max[2]
}

fn extract_tile_asset(asset: &NavMeshAsset, tile_id: u32) -> Result<NavMeshAsset, NavigationError> {
    let mut tile_asset = NavMeshAsset::empty(asset.agent_type.clone());
    tile_asset.area_costs = asset.area_costs.clone();
    tile_asset.tiles = asset
        .tiles
        .iter()
        .filter(|tile| tile.id == tile_id)
        .cloned()
        .collect();
    let mut vertex_remap = HashMap::<u32, u32>::new();
    for polygon in asset
        .polygons
        .iter()
        .filter(|polygon| polygon.tile == tile_id)
    {
        let start = polygon.first_index as usize;
        let end = start.saturating_add(polygon.index_count as usize);
        if end > asset.indices.len() {
            return Err(NavigationError::new(
                NavigationErrorKind::BackendFailure,
                "stored navigation tile has an invalid polygon index range",
            ));
        }
        let first_index = tile_asset.indices.len() as u32;
        for source_index in &asset.indices[start..end] {
            let source_vertex = *asset.vertices.get(*source_index as usize).ok_or_else(|| {
                NavigationError::new(
                    NavigationErrorKind::BackendFailure,
                    "stored navigation tile references a missing vertex",
                )
            })?;
            let target = *vertex_remap.entry(*source_index).or_insert_with(|| {
                let index = tile_asset.vertices.len() as u32;
                tile_asset.vertices.push(source_vertex);
                index
            });
            tile_asset.indices.push(target);
        }
        tile_asset.polygons.push(NavMeshPolygonAsset {
            first_index,
            index_count: polygon.index_count,
            area: polygon.area,
            tile: tile_id,
        });
    }
    Ok(tile_asset)
}
