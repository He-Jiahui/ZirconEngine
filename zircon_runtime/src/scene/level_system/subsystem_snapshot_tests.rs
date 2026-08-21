use std::sync::Arc;
use std::time::Instant;

use super::*;

fn test_level() -> LevelSystem {
    LevelSystem::new(
        WorldHandle::new(91),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    )
}

#[test]
fn subsystem_registry_reads_share_one_immutable_snapshot() {
    let level = test_level();
    let empty = level.registered_subsystems();
    assert!(empty.is_empty());

    level.register_subsystem("physics");
    let first = level.registered_subsystems();
    let first_again = level.registered_subsystems();
    assert_eq!(first.as_ref(), ["physics".to_string()].as_slice());
    assert!(Arc::ptr_eq(&first, &first_again));

    level.register_subsystem("animation");
    let second = level.registered_subsystems();
    assert_eq!(
        second.as_ref(),
        ["physics".to_string(), "animation".to_string()].as_slice()
    );
    assert_eq!(first.as_ref(), ["physics".to_string()].as_slice());
    assert!(!Arc::ptr_eq(&first, &second));
}

#[test]
#[ignore = "release-mode subsystem snapshot evidence; run explicitly"]
fn level_subsystem_registry_snapshot_release_benchmark() {
    const SAMPLE_PAIRS: usize = 21;
    const SUBSYSTEMS: usize = 1_024;
    const READS: usize = 4_096;

    let level = test_level();
    for index in 0..SUBSYSTEMS {
        level.register_subsystem(format!("runtime.subsystem.{index:04}"));
    }

    fn run_legacy(level: &LevelSystem) -> u128 {
        let started = Instant::now();
        let mut observed = 0usize;
        for _ in 0..READS {
            let names = level.registered_subsystems().to_vec();
            observed = observed.saturating_add(std::hint::black_box(names).len());
        }
        assert_eq!(observed, SUBSYSTEMS * READS);
        started.elapsed().as_nanos()
    }

    fn run_snapshot(level: &LevelSystem) -> u128 {
        let started = Instant::now();
        let mut observed = 0usize;
        for _ in 0..READS {
            let snapshot = std::hint::black_box(level.registered_subsystems());
            observed = observed.saturating_add(snapshot.len());
        }
        assert_eq!(observed, SUBSYSTEMS * READS);
        started.elapsed().as_nanos()
    }

    fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(percentile).saturating_add(99) / 100;
        samples[rank.saturating_sub(1).min(samples.len().saturating_sub(1))]
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut snapshot_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_first = pair % 2 == 0;
        let first = if legacy_first {
            run_legacy(&level)
        } else {
            run_snapshot(&level)
        };
        let second = if legacy_first {
            run_snapshot(&level)
        } else {
            run_legacy(&level)
        };
        let (legacy, snapshot) = if legacy_first {
            (first, second)
        } else {
            (second, first)
        };
        legacy_samples.push(legacy);
        snapshot_samples.push(snapshot);
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples.clone(), 95);
    let snapshot_p50_ns = nearest_rank(&mut snapshot_samples.clone(), 50);
    let snapshot_p95_ns = nearest_rank(&mut snapshot_samples.clone(), 95);
    let legacy_ns = legacy_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let snapshot_ns = snapshot_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let legacy_string_clones = SUBSYSTEMS.saturating_mul(READS);
    let snapshot_arc_clones = READS;
    let clone_work_reduction_basis_points = legacy_string_clones
        .saturating_sub(snapshot_arc_clones)
        .saturating_mul(10_000)
        / legacy_string_clones;

    assert!(
        snapshot_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "immutable registry snapshot P95 must be at most 25% of full string cloning P95"
    );
    println!(
        "PERF-MVP-RWL-P2-004 sample_pairs={SAMPLE_PAIRS} sample_order=alternating \
percentile_method=nearest_rank subsystems={SUBSYSTEMS} reads={READS} \
legacy_ns={legacy_ns} snapshot_ns={snapshot_ns} legacy_p50_ns={legacy_p50_ns} \
legacy_p95_ns={legacy_p95_ns} snapshot_p50_ns={snapshot_p50_ns} snapshot_p95_ns={snapshot_p95_ns} \
legacy_string_clones={legacy_string_clones} snapshot_arc_clones={snapshot_arc_clones} \
clone_work_reduction_basis_points={clone_work_reduction_basis_points}"
    );
}
