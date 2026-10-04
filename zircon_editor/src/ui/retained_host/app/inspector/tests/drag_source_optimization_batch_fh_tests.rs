use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const REFERENCES_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fh_editor394_object_reference_preserves_bytes() {
    for id in [0, 1, 42, u64::MAX] {
        assert_eq!(scene_node_reference(id), legacy_scene_node_reference(id));
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fh_editor394_direct_object_reference_benchmark() {
    let id = 18_446_744_073_709_551_615;
    for _ in 0..4 {
        black_box(measure(legacy_scene_node_reference, id));
        black_box(measure(scene_node_reference, id));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(legacy_scene_node_reference, id));
            optimized_samples.push(measure(scene_node_reference, id));
        } else {
            optimized_samples.push(measure(scene_node_reference, id));
            legacy_samples.push(measure(legacy_scene_node_reference, id));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn legacy_scene_node_reference(id: u64) -> String {
    format!("object://scene/node/{id}")
}

fn measure(mut build: impl FnMut(u64) -> String, id: u64) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..REFERENCES_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(build(black_box(id))).len());
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
        "EDITOR394_DIRECT_OBJECT_REFERENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} references_per_sample={REFERENCES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
        "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
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
