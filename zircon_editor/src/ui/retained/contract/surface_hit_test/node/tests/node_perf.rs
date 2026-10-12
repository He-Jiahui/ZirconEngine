use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const PATHS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fi_editor395_template_node_path_preserves_bytes() {
    for node_id in ["", "root", "Inspector/Fields/name", "node.with-symbols_42"] {
        assert_eq!(
            template_node_path(node_id).0,
            legacy_template_node_path(node_id).0
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fi_editor395_direct_template_node_path_benchmark() {
    const NODE_ID: &str = "InspectorPanel/Transform/TranslationX";
    for _ in 0..4 {
        black_box(measure(legacy_template_node_path, NODE_ID));
        black_box(measure(template_node_path, NODE_ID));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(legacy_template_node_path, NODE_ID));
            optimized_samples.push(measure(template_node_path, NODE_ID));
        } else {
            optimized_samples.push(measure(template_node_path, NODE_ID));
            legacy_samples.push(measure(legacy_template_node_path, NODE_ID));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn legacy_template_node_path(node_id: &str) -> UiNodePath {
    UiNodePath::new(format!("template_nodes/{node_id}"))
}

fn measure(mut build: impl FnMut(&str) -> UiNodePath, node_id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..PATHS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(build(black_box(node_id))).0.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR395_DIRECT_TEMPLATE_NODE_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "optimized p95 {optimized_p95}ns must be at most 80% of legacy p95 {legacy_p95}ns"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
