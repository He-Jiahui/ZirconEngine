use std::collections::{HashMap, HashSet, VecDeque};
use std::hint::black_box;
use std::time::Instant;

use super::{collect_dependency_readiness, AssetId, AssetLoadState, ResourceReadinessGeneration};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 1_024;
const DEPENDENCIES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fl_runtime207_capacity_preserves_missing_dependency_rows() {
    let generation = ResourceReadinessGeneration::default();
    let root_id = AssetId::new();
    let dependency_ids = (0..DEPENDENCIES_PER_BUILD)
        .map(|_| AssetId::new())
        .collect::<Vec<_>>();

    let rows = collect_dependency_readiness(&generation, root_id, &dependency_ids);

    assert_eq!(rows.len(), DEPENDENCIES_PER_BUILD);
    assert!(rows.capacity() >= dependency_ids.len());
    for (row, dependency_id) in rows.iter().zip(&dependency_ids) {
        assert_eq!(row.id, *dependency_id);
        assert_eq!(row.depth, 1);
        assert!(row.direct);
        assert_eq!(row.load_state, AssetLoadState::Failed);
        assert_eq!(row.diagnostics.len(), 1);
    }
}

#[test]
fn optimization_batch_20260826fl_runtime207_traversal_containers_reserve_direct_dependencies() {
    let source = include_str!("../readiness.rs");
    assert!(source.contains("let initial_capacity = dependency_ids.len();"));
    assert!(source.contains("Vec::with_capacity(initial_capacity)"));
    assert!(source.contains("HashMap::with_capacity(initial_capacity)"));
    assert!(source.contains("HashSet::with_capacity(initial_capacity.saturating_add(1))"));
    assert!(source.contains("VecDeque::with_capacity(initial_capacity)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fl_runtime207_dependency_readiness_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME207_DEPENDENCY_READINESS_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} dependencies_per_build={DEPENDENCIES_PER_BUILD} \
container_count=4 legacy_reservations_per_container=0 optimized_reservations_per_container=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut rows = if reserve {
            Vec::with_capacity(DEPENDENCIES_PER_BUILD)
        } else {
            Vec::new()
        };
        let mut row_by_id = if reserve {
            HashMap::with_capacity(DEPENDENCIES_PER_BUILD)
        } else {
            HashMap::new()
        };
        let mut expanded = if reserve {
            HashSet::with_capacity(DEPENDENCIES_PER_BUILD + 1)
        } else {
            HashSet::new()
        };
        let mut queue = if reserve {
            VecDeque::with_capacity(DEPENDENCIES_PER_BUILD)
        } else {
            VecDeque::new()
        };
        expanded.insert(usize::MAX);
        for dependency in 0..DEPENDENCIES_PER_BUILD {
            queue.push_back(black_box(dependency));
        }
        while let Some(dependency) = queue.pop_front() {
            row_by_id.insert(black_box(dependency), rows.len());
            expanded.insert(black_box(dependency));
            rows.push(black_box([dependency; 8]));
        }
        checksum ^= black_box(
            rows.len()
                ^ rows.capacity()
                ^ row_by_id.len()
                ^ row_by_id.capacity()
                ^ expanded.len()
                ^ expanded.capacity()
                ^ queue.capacity(),
        );
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
