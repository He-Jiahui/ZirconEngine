use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionState, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBuffer64Entry,
};

use super::{
    collect_and_pack_execution_visbuffer64_entries, collect_execution_visbuffer64_entries,
    pack_execution_visbuffer64_entries,
};

const BENCH_CLUSTER_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn single_pass_visbuffer_projection_matches_typed_and_packed_outputs() {
    let selected_clusters = benchmark_selected_clusters();

    assert_eq!(
        collect_and_pack_execution_visbuffer64_entries(&selected_clusters),
        legacy_projection(&selected_clusters)
    );
}

#[test]
#[ignore = "release-only VisBuffer64 projection benchmark"]
fn visbuffer64_single_pass_projection_release_benchmark_evidence() {
    let selected_clusters = benchmark_selected_clusters();
    assert_eq!(
        collect_and_pack_execution_visbuffer64_entries(&selected_clusters),
        legacy_projection(&selected_clusters)
    );

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_legacy(&selected_clusters),
        || measure_optimized(&selected_clusters),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=visbuffer64_single_pass_projection \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
selected_cluster_count={BENCH_CLUSTER_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_source_scans=2 optimized_source_scans=1 optimized_vectors_preallocated=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "single-pass VisBuffer64 projection must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_projection(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> (Vec<RenderVirtualGeometryVisBuffer64Entry>, Vec<u64>) {
    let entries = collect_execution_visbuffer64_entries(selected_clusters);
    let packed_words = pack_execution_visbuffer64_entries(&entries);
    (entries, packed_words)
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_legacy(selected_clusters: &[RenderVirtualGeometrySelectedCluster]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_projection(black_box(selected_clusters)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(selected_clusters: &[RenderVirtualGeometrySelectedCluster]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(collect_and_pack_execution_visbuffer64_entries(black_box(
            selected_clusters,
        )));
    }
    started.elapsed().as_nanos().max(1)
}

fn benchmark_selected_clusters() -> Vec<RenderVirtualGeometrySelectedCluster> {
    (0..BENCH_CLUSTER_COUNT as u32)
        .map(|cluster_id| RenderVirtualGeometrySelectedCluster {
            instance_index: Some(cluster_id % 64),
            entity: u64::from(cluster_id % 512),
            cluster_id,
            cluster_ordinal: cluster_id % 32,
            page_id: cluster_id / 4,
            lod_level: (cluster_id % 8) as u8,
            state: RenderVirtualGeometryExecutionState::Resident,
        })
        .collect()
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
