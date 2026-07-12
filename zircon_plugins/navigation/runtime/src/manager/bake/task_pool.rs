use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_plugin_navigation_recast::{merge_tiled_assets, RecastTiledBakePlan};
use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeDiagnostic, NavMeshBakeReport, NavMeshBakeRequest, NavigationError,
    NavigationErrorKind,
};
use zircon_runtime::scene::World;

use super::{
    bake_runtime_counts, canonical_surface_key, finish_bake, prepare_bake, tiled, BakePreparation,
};
use crate::manager::DefaultNavigationManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NavMeshBakeTaskHandle(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavMeshBakeTaskState {
    Pending,
    Ready,
}

#[derive(Debug)]
enum PendingTiledBakeState {
    Preparing,
    Baking {
        completed: usize,
        results: Vec<Option<Result<NavMeshAsset, NavigationError>>>,
        preparation: BakePreparation,
        world: World,
        plan: RecastTiledBakePlan,
        generation: u64,
    },
    Failed(NavigationError),
}

#[derive(Clone, Debug)]
pub(in crate::manager) struct PendingTiledBake {
    shared: Arc<Mutex<PendingTiledBakeState>>,
    surface_entity: Option<u64>,
}

impl PendingTiledBake {
    fn is_ready(&self) -> bool {
        let state = lock_task(&self.shared);
        match &*state {
            PendingTiledBakeState::Preparing => false,
            PendingTiledBakeState::Baking {
                completed, results, ..
            } => *completed == results.len(),
            PendingTiledBakeState::Failed(_) => true,
        }
    }

    pub(in crate::manager) fn surface_entity(&self) -> Option<u64> {
        self.surface_entity
    }
}

fn lock_task(shared: &Mutex<PendingTiledBakeState>) -> MutexGuard<'_, PendingTiledBakeState> {
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn panic_error(context: &str) -> NavigationError {
    NavigationError::new(
        NavigationErrorKind::BackendFailure,
        format!("navigation {context} panicked"),
    )
}

impl DefaultNavigationManager {
    pub fn start_tiled_bake(
        &self,
        world: World,
        request: NavMeshBakeRequest,
    ) -> NavMeshBakeTaskHandle {
        let surface_entity = canonical_surface_key(&world, request.surface_entity);
        let shared = Arc::new(Mutex::new(PendingTiledBakeState::Preparing));
        let (generation, handle) = {
            let mut state = self.lock_state();
            let generation = state.advance_bake_context(surface_entity);
            let handle = NavMeshBakeTaskHandle(state.next_bake_task);
            state.next_bake_task = state.next_bake_task.saturating_add(1);
            state.bake_tasks.insert(
                handle,
                PendingTiledBake {
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
                manager.prepare_tiled_task(world, request, generation, shared_for_work);
            }))
            .is_err()
            {
                *lock_task(&shared) =
                    PendingTiledBakeState::Failed(panic_error("tile preparation"));
            }
        });
        handle
    }

    pub fn bake_task_state(&self, handle: NavMeshBakeTaskHandle) -> Option<NavMeshBakeTaskState> {
        let state = self.lock_state();
        state
            .bake_tasks
            .get(&handle)
            .map(PendingTiledBake::is_ready)
            .or_else(|| {
                state
                    .dirty_bake_tasks
                    .get(&handle)
                    .map(super::dirty::PendingDirtyBake::is_ready)
            })
            .map(|ready| {
                if ready {
                    NavMeshBakeTaskState::Ready
                } else {
                    NavMeshBakeTaskState::Pending
                }
            })
    }

    pub fn try_harvest_tiled_bake(
        &self,
        handle: NavMeshBakeTaskHandle,
    ) -> Option<Result<NavMeshBakeReport, NavigationError>> {
        let task = {
            let mut state = self.lock_state();
            if !state.bake_tasks.get(&handle)?.is_ready() {
                return None;
            }
            state.bake_tasks.remove(&handle)?
        };
        Some(self.finish_tiled_task(task))
    }

    pub fn bake_diagnostics(&self) -> Vec<NavMeshBakeDiagnostic> {
        self.lock_state().bake_diagnostics.clone()
    }

    fn prepare_tiled_task(
        &self,
        world: World,
        request: NavMeshBakeRequest,
        generation: u64,
        shared: Arc<Mutex<PendingTiledBakeState>>,
    ) {
        let prepared = prepare_bake(self, &world, request).and_then(|preparation| {
            let plan = tiled::plan_for_preparation(self, &preparation)?.ok_or_else(|| {
                NavigationError::new(
                    NavigationErrorKind::InvalidConfiguration,
                    "asynchronous tiled bake requires a surface override_tile_size and source geometry",
                )
            })?;
            Ok((preparation, plan))
        });
        let (preparation, plan) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                *lock_task(&shared) = PendingTiledBakeState::Failed(error);
                return;
            }
        };
        let tiles = plan.tiles().to_vec();
        *lock_task(&shared) = PendingTiledBakeState::Baking {
            completed: 0,
            results: std::iter::repeat_with(|| None).take(tiles.len()).collect(),
            preparation,
            world,
            plan: plan.clone(),
            generation,
        };

        for (index, tile) in tiles.into_iter().enumerate() {
            let shared = Arc::clone(&shared);
            let plan = plan.clone();
            let backend = self.backend.clone();
            self.bake_pool.spawn(move || {
                let result =
                    catch_unwind(AssertUnwindSafe(|| backend.bake_planned_tile(&plan, tile)))
                        .unwrap_or_else(|_| Err(panic_error("tile worker")));
                let mut state = lock_task(&shared);
                if let PendingTiledBakeState::Baking {
                    completed, results, ..
                } = &mut *state
                {
                    results[index] = Some(result);
                    *completed += 1;
                }
            });
        }
    }

    fn finish_tiled_task(
        &self,
        task: PendingTiledBake,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        let context_surface = task.surface_entity;
        let state = std::mem::replace(
            &mut *lock_task(&task.shared),
            PendingTiledBakeState::Preparing,
        );
        let (mut results, preparation, world, plan, generation) = match state {
            PendingTiledBakeState::Baking {
                results,
                preparation,
                world,
                plan,
                generation,
                ..
            } => (results, preparation, world, plan, generation),
            PendingTiledBakeState::Failed(error) => return Err(error),
            PendingTiledBakeState::Preparing => {
                return Err(NavigationError::new(
                    NavigationErrorKind::BackendFailure,
                    "navigation tiled bake was harvested before completion",
                ));
            }
        };
        let mut assets = Vec::with_capacity(results.len());
        for result in &mut results {
            assets.push(result.take().expect("completed tile result missing")?);
        }
        let mut asset = merge_tiled_assets(preparation.agent_type.clone(), assets)?;
        let identity = preparation.tiled_identity();
        let report = finish_bake(&world, preparation, &mut asset);
        self.publish_bake(
            context_surface,
            generation,
            Some((identity, plan, asset)),
            report.diagnostics.clone(),
            bake_runtime_counts(&world),
        )?;
        Ok(report)
    }
}
