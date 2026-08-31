use std::hint::black_box;
use std::time::Instant;

use super::reflection_snapshot_node_capacity_from_count;

const SAMPLE_PAIRS: usize = 21;
const SNAPSHOTS_PER_SAMPLE: usize = 128;
const NODES_PER_SNAPSHOT: usize = 4_096;

#[test]
fn optimization_batch_20260826go_runtime235_capacity_matches_tree_node_count() {
    assert_eq!(reflection_snapshot_node_capacity_from_count(0), 0);
    assert_eq!(reflection_snapshot_node_capacity_from_count(4_096), 4_096);
    assert_eq!(
        reflection_snapshot_node_capacity_from_count(usize::MAX),
        usize::MAX
    );
}

#[test]
fn optimization_batch_20260826go_runtime235_snapshot_preallocates_tree_nodes() {
    let source = include_str!("../reflection_snapshot.rs");

    assert!(source.contains("Vec::with_capacity(reflection_snapshot_node_capacity(surface))"));
    assert!(source.contains("surface.tree.nodes.len()"));
    assert!(!source.contains("let mut nodes = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826go_runtime235_reflection_snapshot_node_capacity_bench() {
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
        "RUNTIME235_REFLECTION_SNAPSHOT_NODE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
snapshots_per_sample={SNAPSHOTS_PER_SAMPLE} nodes_per_snapshot={NODES_PER_SNAPSHOT} \
node_payload_usize_fields=16 legacy_initial_capacity=0 \
optimized_initial_capacity={NODES_PER_SNAPSHOT} legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(preallocate: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for snapshot in 0..SNAPSHOTS_PER_SAMPLE {
        let mut nodes = if preallocate {
            Vec::with_capacity(NODES_PER_SNAPSHOT)
        } else {
            Vec::new()
        };
        for node in 0..NODES_PER_SNAPSHOT {
            let value = black_box(snapshot * NODES_PER_SNAPSHOT + node);
            nodes.push([value; 16]);
        }
        checksum ^= black_box(nodes.len() ^ nodes.capacity() ^ snapshot);
        black_box(nodes);
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
