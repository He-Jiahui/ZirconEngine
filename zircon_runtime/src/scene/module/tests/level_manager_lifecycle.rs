use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::core::framework::scene::{WorldHandle, SCENE_MODULE_NAME};
use crate::core::{CoreError, CoreRuntime, CoreWeak, InitLevel, ModuleDescriptor};
use crate::scene::{Resource, World, DEFAULT_LEVEL_MANAGER_NAME};

use super::{DefaultLevelManager, PreparedLevel, PreparedLevelPublishError};

#[test]
fn level_handle_allocation_accepts_the_maximum_once_then_reports_exhaustion() {
    let manager = DefaultLevelManager::default();
    manager.next_handle.store(u64::MAX - 1, Ordering::Relaxed);

    let last_level = manager.try_create_default_level().unwrap();
    assert_eq!(last_level.handle(), WorldHandle::new(u64::MAX));
    assert!(manager.level(last_level.handle()).is_some());
    assert!(matches!(
        manager.try_create_default_level(),
        Err(CoreError::LevelHandleExhausted)
    ));
    assert!(manager.level(WorldHandle::new(0)).is_none());
}

#[test]
fn level_manager_registry_orders_world_snapshots_by_handle() {
    let manager = DefaultLevelManager::default();
    manager.next_handle.store(40, Ordering::Relaxed);
    manager.try_create_default_level().unwrap();
    manager.try_create_default_level().unwrap();
    manager.try_create_default_level().unwrap();

    let handles = manager
        .level_snapshots_in_handle_order()
        .into_iter()
        .map(|level| level.handle().get())
        .collect::<Vec<_>>();

    assert_eq!(handles, vec![41, 42, 43]);
}

fn registered_scene_runtime() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    let descriptor = crate::scene::module_descriptor();
    for dependency in &descriptor.module_dependencies {
        runtime
            .register_module(
                ModuleDescriptor::new(&dependency.module_name, "Scene publication test dependency")
                    .with_init_level(InitLevel::Kernel),
            )
            .unwrap();
    }
    runtime.register_module(descriptor).unwrap();
    runtime.activate_registered_modules().unwrap();
    runtime
}

#[derive(Debug, PartialEq, Eq)]
struct ManagerSnapshot {
    index: u32,
    generation: u32,
    in_flight_calls: usize,
    admission_open: bool,
    handles: Vec<u64>,
}

fn manager_snapshot(runtime: &CoreRuntime) -> ManagerSnapshot {
    let core = runtime.handle();
    let services = core
        .inner
        .services
        .try_lock()
        .expect("install must not hold the service registry lock");
    let entry = services.get(DEFAULT_LEVEL_MANAGER_NAME).unwrap();
    let manager = entry
        .instance
        .as_ref()
        .unwrap()
        .downcast_ref::<DefaultLevelManager>()
        .unwrap();
    let handles = manager
        .levels
        .try_lock()
        .expect("install must not hold the levels registry lock")
        .keys()
        .map(|handle| handle.get())
        .collect();
    ManagerSnapshot {
        index: entry.index,
        generation: entry.generation,
        in_flight_calls: entry.in_flight_calls,
        admission_open: entry.admission_open,
        handles,
    }
}

type RollbackObservations = Arc<Mutex<Vec<Option<(usize, bool, bool)>>>>;

#[derive(Debug)]
struct RollbackWorldResource {
    core: CoreWeak,
    handle: Arc<AtomicU64>,
    observations: RollbackObservations,
}

impl Resource for RollbackWorldResource {}

impl Drop for RollbackWorldResource {
    fn drop(&mut self) {
        // Observe the actual registry without resolving a service or retaining a
        // World clone. Lock contention is recorded as failure, never as success.
        let observation = self.core.upgrade().and_then(|core| {
            let services = core.inner.services.try_lock().ok()?;
            let entry = services.get(DEFAULT_LEVEL_MANAGER_NAME)?;
            let manager = entry
                .instance
                .as_ref()?
                .downcast_ref::<DefaultLevelManager>()?;
            let levels = manager.levels.try_lock().ok()?;
            Some((
                entry.in_flight_calls,
                entry.admission_open,
                levels.contains_key(&WorldHandle::new(self.handle.load(Ordering::Relaxed))),
            ))
        });
        self.observations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(observation);
    }
}

fn prepare_observed_level(runtime: &CoreRuntime) -> (PreparedLevel, u64, RollbackObservations) {
    let observations = Arc::new(Mutex::new(Vec::new()));
    let handle = Arc::new(AtomicU64::new(0));
    let mut world = World::empty();
    world.insert_resource(RollbackWorldResource {
        core: runtime.handle().downgrade(),
        handle: Arc::clone(&handle),
        observations: Arc::clone(&observations),
    });
    let prepared =
        crate::scene::prepare_level(&runtime.handle(), world, Default::default()).unwrap();
    let prepared_handle = prepared.level.as_ref().unwrap().handle().get();
    handle.store(prepared_handle, Ordering::Relaxed);
    (prepared, prepared_handle, observations)
}

#[test]
fn prepared_level_publication_rolls_back_until_committed() {
    let runtime = registered_scene_runtime();
    let (first, first_handle, observations) = prepare_observed_level(&runtime);
    assert_eq!(manager_snapshot(&runtime).in_flight_calls, 0);
    assert!(manager_snapshot(&runtime).handles.is_empty());

    let result = first.try_publish_with(|level| {
        assert_eq!(level.handle().get(), first_handle);
        let snapshot = manager_snapshot(&runtime);
        assert_eq!(snapshot.in_flight_calls, 1);
        assert_eq!(snapshot.handles, vec![first_handle]);
        Err("original install failure")
    });
    assert!(matches!(
        result,
        Err(PreparedLevelPublishError::Install(
            "original install failure"
        ))
    ));
    assert_eq!(*observations.lock().unwrap(), vec![Some((1, true, false))]);
    assert_eq!(manager_snapshot(&runtime).in_flight_calls, 0);
    assert!(manager_snapshot(&runtime).handles.is_empty());

    let second =
        crate::scene::prepare_level(&runtime.handle(), World::empty(), Default::default()).unwrap();
    let second_handle = second.level.as_ref().unwrap().handle().get();
    let committed = second
        .try_publish_with(|level| {
            assert_eq!(level.handle().get(), second_handle);
            assert_eq!(manager_snapshot(&runtime).handles, vec![second_handle]);
            Ok::<(), &str>(())
        })
        .unwrap();
    assert_eq!(committed.handle().get(), second_handle);
    assert_eq!(manager_snapshot(&runtime).in_flight_calls, 0);
    assert_eq!(manager_snapshot(&runtime).handles, vec![second_handle]);
}

#[test]
fn prepared_level_install_panic_releases_transient_world_before_admission() {
    let runtime = registered_scene_runtime();
    let (prepared, handle, observations) = prepare_observed_level(&runtime);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _: Result<_, PreparedLevelPublishError<()>> = prepared.try_publish_with(|level| {
            assert_eq!(level.handle().get(), handle);
            assert_eq!(manager_snapshot(&runtime).handles, vec![handle]);
            panic!("original install panic");
        });
    }))
    .expect_err("the install panic must remain visible");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"original install panic")
    );
    assert_eq!(*observations.lock().unwrap(), vec![Some((1, true, false))]);
    assert!(manager_snapshot(&runtime).handles.is_empty());
    assert_eq!(manager_snapshot(&runtime).in_flight_calls, 0);
}

#[test]
fn prepared_level_idle_seed_rejects_closed_and_reactivated_generation() {
    let runtime = registered_scene_runtime();
    let closed =
        crate::scene::prepare_level(&runtime.handle(), World::empty(), Default::default()).unwrap();
    let stale =
        crate::scene::prepare_level(&runtime.handle(), World::empty(), Default::default()).unwrap();
    let before = manager_snapshot(&runtime);
    assert_eq!(before.in_flight_calls, 0);
    assert!(before.handles.is_empty());
    runtime
        .deactivate_module_with_drain_timeout(SCENE_MODULE_NAME, Duration::ZERO)
        .unwrap();

    let mut installs = 0;
    let closed_result = closed.try_publish_with(|_| {
        installs += 1;
        Ok::<(), ()>(())
    });
    assert!(matches!(
        closed_result,
        Err(PreparedLevelPublishError::Admission(
            CoreError::StaleServiceHandle { .. }
        ))
    ));
    assert_eq!(installs, 0);

    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let after = manager_snapshot(&runtime);
    assert_eq!(after.index, before.index);
    assert_ne!(after.generation, before.generation);
    let stale_result = stale.try_publish_with(|_| {
        installs += 1;
        Ok::<(), ()>(())
    });
    assert!(matches!(
        stale_result,
        Err(PreparedLevelPublishError::Admission(
            CoreError::StaleServiceHandle { .. }
        ))
    ));
    assert_eq!(installs, 0);
    assert!(manager_snapshot(&runtime).handles.is_empty());
    assert_eq!(manager_snapshot(&runtime).in_flight_calls, 0);
}

#[test]
fn prepared_level_rollback_finishes_before_closed_generation_cleanup_retry() {
    let runtime = registered_scene_runtime();
    let (prepared, handle, observations) = prepare_observed_level(&runtime);
    let before = manager_snapshot(&runtime);
    let result = prepared.try_publish_with(|_| {
        assert!(matches!(
            runtime.deactivate_module_with_drain_timeout(SCENE_MODULE_NAME, Duration::ZERO),
            Err(CoreError::ServiceCallDrainTimeout { .. })
        ));
        let stopping = manager_snapshot(&runtime);
        assert_eq!(stopping.index, before.index);
        assert_eq!(stopping.generation, before.generation);
        assert!(!stopping.admission_open);
        assert_eq!(stopping.in_flight_calls, 1);
        assert_eq!(stopping.handles, vec![handle]);
        Err("original install failure after close")
    });
    assert!(matches!(
        result,
        Err(PreparedLevelPublishError::Install(
            "original install failure after close"
        ))
    ));
    assert_eq!(*observations.lock().unwrap(), vec![Some((1, false, false))]);
    let rolled_back = manager_snapshot(&runtime);
    assert_eq!(rolled_back.index, before.index);
    assert_eq!(rolled_back.generation, before.generation);
    assert!(!rolled_back.admission_open);
    assert_eq!(rolled_back.in_flight_calls, 0);
    assert!(rolled_back.handles.is_empty());
    runtime
        .deactivate_module_with_drain_timeout(SCENE_MODULE_NAME, Duration::ZERO)
        .unwrap();
    let core = runtime.handle();
    let services = core.inner.services.lock().unwrap();
    let settled = services.get(DEFAULT_LEVEL_MANAGER_NAME).unwrap();
    assert!(settled.instance.is_none());
    assert_eq!(settled.in_flight_calls, 0);
    assert!(!settled.admission_open);
    assert_ne!(settled.generation, before.generation);
}

#[test]
fn optimization_batch_ic_runtime612_level_registry_is_intrinsically_ordered() {
    let manager_source = include_str!("../default_level_manager.rs");
    let lifecycle_source = include_str!("../level_manager_lifecycle.rs");
    let retired_sort = ["sort_levels", "_by_handle"].concat();

    assert!(manager_source.contains("Mutex<BTreeMap<WorldHandle, LevelSystem>>"));
    assert!(!lifecycle_source.contains(&retired_sort));
    assert!(lifecycle_source.contains("lock_levels().values().cloned().collect"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ic_runtime612_ordered_level_snapshot_p95() {
    const LEVELS: usize = 4_096;
    const ITERATIONS: usize = 64;
    const SAMPLE_PAIRS: usize = 17;
    let mut hash_levels = HashMap::with_capacity(LEVELS);
    let mut ordered_levels = BTreeMap::new();
    for index in 0..LEVELS {
        let handle = (index * 2_053) % LEVELS;
        hash_levels.insert(handle, handle);
        ordered_levels.insert(handle, handle);
    }
    let expected = ordered_levels.values().copied().collect::<Vec<_>>();
    let mut retired = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            retired.push(measure_level_snapshot(&hash_levels, ITERATIONS));
            optimized.push(measure_ordered_level_snapshot(&ordered_levels, ITERATIONS));
        } else {
            optimized.push(measure_ordered_level_snapshot(&ordered_levels, ITERATIONS));
            retired.push(measure_level_snapshot(&hash_levels, ITERATIONS));
        }
    }
    assert_eq!(retired_level_snapshot(&hash_levels), expected);
    assert_eq!(ordered_level_snapshot(&ordered_levels), expected);
    let retired_p95_ns = percentile(&retired, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME612_ORDERED_LEVEL_SNAPSHOT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             iterations={ITERATIONS} levels={LEVELS} retired_p95_ns={retired_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} retired_raw_ns={} optimized_raw_ns={}",
        csv(&retired),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(50),
        "ordered snapshot P95 must be at most 50% of clone-and-sort: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn retired_level_snapshot(levels: &HashMap<usize, usize>) -> Vec<usize> {
    let mut snapshot = levels.values().copied().collect::<Vec<_>>();
    snapshot.sort_unstable();
    snapshot
}

fn ordered_level_snapshot(levels: &BTreeMap<usize, usize>) -> Vec<usize> {
    levels.values().copied().collect()
}

fn measure_level_snapshot(levels: &HashMap<usize, usize>, iterations: usize) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(retired_level_snapshot(black_box(levels)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_ordered_level_snapshot(levels: &BTreeMap<usize, usize>, iterations: usize) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(ordered_level_snapshot(black_box(levels)));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
