use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionState, RenderVirtualGeometrySelectedCluster,
};

use super::super::build_records::{
    refresh_seed_backed_frontier_ranks, seed_backed_record_sort_key,
};
use super::super::record::SeedBackedExecutionSelectionRecord;
use super::finalize_seed_backed_execution_records;
use crate::virtual_geometry::types::{
    VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
};

const BENCH_RECORD_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn seed_execution_record_finalization_preserves_sort_budget_and_frontier_ranks() {
    let records = vec![record(30, 3), record(10, 1), record(20, 2)];
    let mut optimized = records.clone();
    let mut legacy = records;

    finalize_seed_backed_execution_records(&mut optimized, 2);
    legacy_finalize(&mut legacy, 2);

    assert_eq!(optimized, legacy);
    assert_eq!(
        optimized
            .iter()
            .map(|record| record.selected_cluster().cluster_id)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
}

#[test]
#[ignore = "release-only seed execution record finalization benchmark"]
fn seed_execution_record_finalization_release_benchmark_evidence() {
    let records = (0..BENCH_RECORD_COUNT)
        .map(|index| {
            let shuffled = (index * 1_549) % BENCH_RECORD_COUNT;
            record(shuffled as u32, shuffled as u32)
        })
        .collect::<Vec<_>>();
    let cluster_budget = BENCH_RECORD_COUNT / 2;
    let mut optimized = records.clone();
    let mut legacy = records.clone();
    finalize_seed_backed_execution_records(&mut optimized, cluster_budget);
    legacy_finalize(&mut legacy, cluster_budget);
    assert_eq!(optimized, legacy);

    for _ in 0..4 {
        black_box(measure_legacy(&records, cluster_budget));
        black_box(measure_optimized(&records, cluster_budget));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&records, cluster_budget));
            optimized_samples.push(measure_optimized(&records, cluster_budget));
        } else {
            optimized_samples.push(measure_optimized(&records, cluster_budget));
            legacy_samples.push(measure_legacy(&records, cluster_budget));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=seed_execution_record_finalize \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
record_count={BENCH_RECORD_COUNT} cluster_budget={cluster_budget} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_sort=stable_with_scratch optimized_sort=unstable_in_place \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(19),
        "in-place seed record finalization must reduce P95 by at least 5%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(records: &[SeedBackedExecutionSelectionRecord], budget: usize) -> u128 {
    let mut inputs = (0..CHECKS_PER_SAMPLE)
        .map(|_| records.to_vec())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for input in &mut inputs {
        legacy_finalize(input, budget);
        black_box(&*input);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(records: &[SeedBackedExecutionSelectionRecord], budget: usize) -> u128 {
    let mut inputs = (0..CHECKS_PER_SAMPLE)
        .map(|_| records.to_vec())
        .collect::<Vec<_>>();
    let started = Instant::now();
    for input in &mut inputs {
        finalize_seed_backed_execution_records(input, budget);
        black_box(&*input);
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_finalize(records: &mut Vec<SeedBackedExecutionSelectionRecord>, budget: usize) {
    records.sort_by_key(seed_backed_record_sort_key);
    if records.len() > budget {
        records.truncate(budget);
    }
    refresh_seed_backed_frontier_ranks(records);
}

fn record(cluster_id: u32, ordinal: u32) -> SeedBackedExecutionSelectionRecord {
    let state = if cluster_id % 2 == 0 {
        VirtualGeometryPrepareClusterState::PendingUpload
    } else {
        VirtualGeometryPrepareClusterState::Resident
    };
    let execution_state = if cluster_id % 2 == 0 {
        RenderVirtualGeometryExecutionState::PendingUpload
    } else {
        RenderVirtualGeometryExecutionState::Resident
    };
    SeedBackedExecutionSelectionRecord::new(
        VirtualGeometryClusterSelection {
            submission_index: cluster_id,
            instance_index: Some(0),
            entity: 42,
            cluster_id,
            cluster_ordinal: ordinal,
            page_id: cluster_id,
            lod_level: 0,
            submission_page_id: cluster_id,
            submission_lod_level: 0,
            entity_cluster_start_ordinal: ordinal as usize,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: BENCH_RECORD_COUNT,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state,
        },
        RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: 42,
            cluster_id,
            cluster_ordinal: ordinal,
            page_id: cluster_id,
            lod_level: 0,
            state: execution_state,
        },
    )
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
