use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use crate::core::framework::scene::WorldHandle;
use crate::core::CoreError;

use super::{DefaultLevelManager, PreparedLevel};

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

#[test]
fn prepared_level_publication_rolls_back_until_committed() {
    let manager = Arc::new(DefaultLevelManager::default());
    let first = manager
        .try_prepare_level(crate::scene::World::empty(), Default::default())
        .unwrap();
    let first_handle = first.handle();
    let first = PreparedLevel::new(Arc::clone(&manager), first);
    assert!(manager.level(first_handle).is_none());

    let publication = first.publish();
    assert!(manager.level(first_handle).is_some());
    drop(publication);
    assert!(manager.level(first_handle).is_none());

    let second = manager
        .try_prepare_level(crate::scene::World::empty(), Default::default())
        .unwrap();
    let second_handle = second.handle();
    let publication = PreparedLevel::new(Arc::clone(&manager), second).publish();
    let committed = publication.commit();
    assert_eq!(committed.handle(), second_handle);
    assert!(manager.level(second_handle).is_some());
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
