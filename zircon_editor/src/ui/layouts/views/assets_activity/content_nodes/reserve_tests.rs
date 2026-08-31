use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const PROJECTIONS_PER_SAMPLE: usize = 64;
const NODES_PER_PROJECTION: usize = 4_096;

#[test]
fn optimization_batch_20260826gm_editor179_content_node_count_matches_rows_and_empty_state() {
    assert_eq!(assets_activity_content_node_count(0, 0), 1);
    assert_eq!(assets_activity_content_node_count(3, 0), 15);
    assert_eq!(assets_activity_content_node_count(0, 4), 20);
    assert_eq!(assets_activity_content_node_count(3, 4), 35);
}

#[test]
fn optimization_batch_20260826gm_editor179_content_nodes_reserve_exact_append_count() {
    let source = include_str!("../content_nodes.rs");

    assert!(source.contains("const CONTENT_NODES_PER_ROW: usize = 5;"));
    assert!(source.contains("nodes.reserve(assets_activity_content_node_count("));
    assert!(source.contains("snapshot.visible_folders.len()"));
    assert!(source.contains("snapshot.visible_assets.len()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gm_editor179_asset_content_node_reserve_bench() {
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
        "EDITOR179_ASSET_CONTENT_NODE_RESERVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
projections_per_sample={PROJECTIONS_PER_SAMPLE} nodes_per_projection={NODES_PER_PROJECTION} \
node_payload_usize_fields=8 legacy_preallocated_nodes_per_projection=0 \
optimized_preallocated_nodes_per_projection={NODES_PER_PROJECTION} \
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
    for projection in 0..PROJECTIONS_PER_SAMPLE {
        let mut nodes = if reserve {
            Vec::with_capacity(NODES_PER_PROJECTION)
        } else {
            Vec::new()
        };
        for node in 0..NODES_PER_PROJECTION {
            let value = black_box(projection * NODES_PER_PROJECTION + node);
            nodes.push([value; 8]);
        }
        checksum ^= black_box(nodes.len() ^ nodes.capacity() ^ projection);
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
