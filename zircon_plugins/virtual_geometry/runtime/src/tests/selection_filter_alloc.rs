use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::collect_execution_cluster_selections_from_submission_keys;
use crate::virtual_geometry::types::{
    VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
};

const BENCH_SELECTION_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn execution_selection_filter_preserves_first_cluster_and_total_order() {
    let selections = [
        selection(2, 30, 3, 300),
        selection(1, 20, 2, 200),
        selection(1, 20, 9, 900),
        selection(1, 10, 1, 100),
        selection(7, 70, 7, 700),
    ];
    let executed_submission_keys = HashSet::from([(42, 1), (42, 2)]);

    let optimized = collect_execution_cluster_selections_from_submission_keys(
        Some(&selections),
        &executed_submission_keys,
    );
    let legacy = legacy_collect(Some(&selections), &executed_submission_keys);

    assert_eq!(optimized, legacy);
    assert_eq!(
        optimized
            .iter()
            .map(|selection| (selection.cluster_id, selection.page_id))
            .collect::<Vec<_>>(),
        vec![(10, 100), (20, 200), (30, 300)]
    );
}

#[test]
#[ignore = "release-only executed selection filter benchmark"]
fn execution_selection_filter_release_benchmark_evidence() {
    let selections = (0..BENCH_SELECTION_COUNT)
        .map(|index| {
            let shuffled = (index * 1_549) % BENCH_SELECTION_COUNT;
            selection(
                shuffled as u32,
                shuffled as u32,
                shuffled as u32,
                shuffled as u32,
            )
        })
        .collect::<Vec<_>>();
    let executed_submission_keys = selections
        .iter()
        .map(|selection| (selection.entity, selection.submission_index))
        .collect::<HashSet<_>>();
    assert_eq!(
        legacy_collect(Some(&selections), &executed_submission_keys),
        collect_execution_cluster_selections_from_submission_keys(
            Some(&selections),
            &executed_submission_keys,
        )
    );

    for _ in 0..4 {
        black_box(measure_legacy(&selections, &executed_submission_keys));
        black_box(measure_optimized(&selections, &executed_submission_keys));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&selections, &executed_submission_keys));
            optimized_samples.push(measure_optimized(&selections, &executed_submission_keys));
        } else {
            optimized_samples.push(measure_optimized(&selections, &executed_submission_keys));
            legacy_samples.push(measure_legacy(&selections, &executed_submission_keys));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=execution_selection_filter_sort \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
selection_count={BENCH_SELECTION_COUNT} executed_key_count={BENCH_SELECTION_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_sort=stable_with_scratch optimized_sort=unstable_in_place \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "in-place execution-selection sorting must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    selections: &[VirtualGeometryClusterSelection],
    executed_submission_keys: &HashSet<(u64, u32)>,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_collect(
            Some(black_box(selections)),
            black_box(executed_submission_keys),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    selections: &[VirtualGeometryClusterSelection],
    executed_submission_keys: &HashSet<(u64, u32)>,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(collect_execution_cluster_selections_from_submission_keys(
            Some(black_box(selections)),
            black_box(executed_submission_keys),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_collect(
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    executed_submission_keys: &HashSet<(u64, u32)>,
) -> Vec<VirtualGeometryClusterSelection> {
    let Some(cluster_selections) = cluster_selections else {
        return Vec::new();
    };
    if executed_submission_keys.is_empty() {
        return Vec::new();
    }

    let mut emitted_clusters = HashSet::<(u64, u32)>::new();
    let mut executed_selections = cluster_selections
        .iter()
        .copied()
        .filter(|selection| {
            executed_submission_keys.contains(&(selection.entity, selection.submission_index))
        })
        .filter(|selection| emitted_clusters.insert((selection.entity, selection.cluster_id)))
        .collect::<Vec<_>>();
    executed_selections.sort_by_key(selection_sort_key);
    executed_selections
}

fn selection_sort_key(
    selection: &VirtualGeometryClusterSelection,
) -> (u32, u64, u32, u32, u32, u8, u32) {
    (
        selection.instance_index.unwrap_or(u32::MAX),
        selection.entity,
        selection.cluster_ordinal,
        selection.cluster_id,
        selection.page_id,
        selection.lod_level,
        selection.submission_index,
    )
}

fn selection(
    submission_index: u32,
    cluster_id: u32,
    cluster_ordinal: u32,
    page_id: u32,
) -> VirtualGeometryClusterSelection {
    VirtualGeometryClusterSelection {
        submission_index,
        instance_index: Some(0),
        entity: 42,
        cluster_id,
        cluster_ordinal,
        page_id,
        lod_level: 0,
        submission_page_id: page_id,
        submission_lod_level: 0,
        entity_cluster_start_ordinal: cluster_ordinal as usize,
        entity_cluster_span_count: 1,
        entity_cluster_total_count: BENCH_SELECTION_COUNT,
        lineage_depth: 0,
        frontier_rank: 0,
        resident_slot: Some(0),
        submission_slot: Some(0),
        state: VirtualGeometryPrepareClusterState::Resident,
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
