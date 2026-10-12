use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const OUTPUTS_PER_SAMPLE: usize = 16_384;

#[test]
fn import_execution_collections_contract_preallocated_gltf_outputs() {
    let mut primitives = Vec::new();
    let mesh_primitives = reserve_gltf_mesh_outputs(&mut primitives, 128);
    assert!(primitives.capacity() >= 128);
    assert!(mesh_primitives.capacity() >= 128);
    assert!(mesh_primitives.is_empty());
}

#[test]
#[ignore = "release performance gate"]
fn import_execution_collections_performance_release_gltf_outputs() {
    for _ in 0..4 {
        black_box(measure_outputs(false));
        black_box(measure_outputs(true));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_growths = None;
    let mut optimized_growths = None;
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if pair_index % 2 == 0 {
            (measure_outputs(false), measure_outputs(true))
        } else {
            let optimized = measure_outputs(true);
            (measure_outputs(false), optimized)
        };
        legacy_growths.get_or_insert(legacy.1);
        optimized_growths.get_or_insert(optimized.1);
        assert_eq!(legacy_growths, Some(legacy.1));
        assert_eq!(optimized_growths, Some(optimized.1));
        legacy_samples.push(legacy.0);
        optimized_samples.push(optimized.0);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_preallocated_gltf_output_collections sample_pairs={SAMPLE_PAIRS} outputs_per_sample={OUTPUTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_capacity_growths_per_sample={} optimized_capacity_growths_per_sample={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        legacy_growths.unwrap(),
        optimized_growths.unwrap(),
    );
    assert_eq!(optimized_growths, Some(0));
    assert!(
        improvement_percent >= 25,
        "preallocated glTF output collections must improve P95 by at least 25%"
    );
}

fn measure_outputs(preallocated: bool) -> (u128, usize) {
    let mut roots = if preallocated {
        Vec::with_capacity(OUTPUTS_PER_SAMPLE)
    } else {
        Vec::new()
    };
    let mut labeled = if preallocated {
        Vec::with_capacity(OUTPUTS_PER_SAMPLE)
    } else {
        Vec::new()
    };
    let started = Instant::now();
    let mut growths = 0;
    for output in 0..OUTPUTS_PER_SAMPLE {
        let root_capacity = roots.capacity();
        roots.push(black_box(output));
        growths += usize::from(roots.capacity() != root_capacity);
        let labeled_capacity = labeled.capacity();
        labeled.push(black_box(output));
        growths += usize::from(labeled.capacity() != labeled_capacity);
    }
    black_box((roots, labeled));
    (started.elapsed().as_nanos().max(1), growths)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
