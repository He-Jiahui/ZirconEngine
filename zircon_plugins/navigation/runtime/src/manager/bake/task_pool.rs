use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_plugin_navigation_recast::{merge_tiled_assets, RecastTiledBakePlan};
use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeDiagnostic, NavMeshBakeReport, NavMeshBakeRequest, NavigationError,
    NavigationErrorKind, NavigationGeneratedBakeSnapshot,
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
        dispatch_complete: bool,
        completed: usize,
        results: Vec<Option<Result<NavMeshAsset, NavigationError>>>,
        preparation: BakePreparation,
        world: World,
        plan: Arc<RecastTiledBakePlan>,
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
                dispatch_complete,
                completed,
                results,
                ..
            } => *dispatch_complete && *completed == results.len(),
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
        let plan = Arc::new(plan);
        let tiles = plan.tiles().to_vec();
        *lock_task(&shared) = PendingTiledBakeState::Baking {
            dispatch_complete: false,
            completed: 0,
            results: std::iter::repeat_with(|| None).take(tiles.len()).collect(),
            preparation,
            world,
            plan: Arc::clone(&plan),
            generation,
        };

        for (index, tile) in tiles.into_iter().enumerate() {
            let shared = Arc::clone(&shared);
            let plan = Arc::clone(&plan);
            let backend = self.backend.clone();
            self.bake_pool.spawn(move || {
                let result =
                    catch_unwind(AssertUnwindSafe(|| backend.bake_planned_tile(&plan, tile)))
                        .unwrap_or_else(|_| Err(panic_error("tile worker")));
                drop(plan);
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
        drop(plan);
        let mut state = lock_task(&shared);
        if let PendingTiledBakeState::Baking {
            dispatch_complete, ..
        } = &mut *state
        {
            *dispatch_complete = true;
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
        let plan =
            Arc::try_unwrap(plan).expect("completed tiled bake retained a worker plan reference");
        let mut assets = Vec::with_capacity(results.len());
        for result in &mut results {
            assets.push(result.take().expect("completed tile result missing")?);
        }
        let mut asset = merge_tiled_assets(preparation.agent_type.clone(), assets)?;
        let identity = preparation.tiled_identity();
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
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_plugin_navigation_recast::{
        RecastBackend, RecastBakeMeshInput, RecastTiledBakeInput,
    };

    use super::*;

    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const CLONES_PER_SAMPLE: usize = 200_000;

    #[test]
    #[ignore = "release-only performance evidence"]
    fn tiled_bake_workers_share_one_plan_allocation() {
        let plan = benchmark_plan();
        let shared_plan = Arc::new(plan.clone());
        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || clone_legacy_plan(&plan),
            || clone_shared_plan(&shared_plan),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins14_tiled_bake_shared_plan clones_per_sample={CLONES_PER_SAMPLE} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_arc_refcount_pairs_per_tile=4 optimized_arc_refcount_pairs_per_tile=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 4,
            "shared-plan P95 {optimized_p95}ns must be no more than 80% of cloned-plan P95 {legacy_p95}ns"
        );
    }

    fn benchmark_plan() -> RecastTiledBakePlan {
        RecastBackend::default()
            .prepare_tiled_bake(RecastTiledBakeInput {
                mesh: RecastBakeMeshInput {
                    agent_type: "humanoid".to_string(),
                    vertices: vec![
                        [-2.0, 0.0, -2.0],
                        [2.0, 0.0, -2.0],
                        [2.0, 0.0, 2.0],
                        [-2.0, 0.0, 2.0],
                    ],
                    indices: vec![0, 1, 2, 0, 2, 3],
                    triangle_areas: Vec::new(),
                    default_area: 1,
                },
                tile_size: 1.0,
            })
            .expect("benchmark tiled plan should be valid")
    }

    fn clone_legacy_plan(plan: &RecastTiledBakePlan) -> usize {
        let mut observed_tiles = 0_usize;
        for _ in 0..CLONES_PER_SAMPLE {
            let cloned = black_box(plan.clone());
            observed_tiles = observed_tiles.wrapping_add(black_box(cloned.tiles().len()));
        }
        observed_tiles
    }

    fn clone_shared_plan(plan: &Arc<RecastTiledBakePlan>) -> usize {
        let mut observed_tiles = 0_usize;
        for _ in 0..CLONES_PER_SAMPLE {
            let cloned = black_box(Arc::clone(plan));
            observed_tiles = observed_tiles.wrapping_add(black_box(cloned.tiles().len()));
        }
        observed_tiles
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        black_box(operation());
        started.elapsed().as_nanos()
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
