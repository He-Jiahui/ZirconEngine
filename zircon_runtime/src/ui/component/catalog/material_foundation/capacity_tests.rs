use std::hint::black_box;
use std::time::Instant;

use super::{
    MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT, UiComponentDescriptorRegistry,
    material_editor_foundation_descriptors,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const DESCRIPTORS_PER_BUILD: usize = 256;
const GROUP_SIZES: [usize; MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT] = [
    11, 11, 11, 11, 11, 11, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
    10,
];

#[test]
fn optimization_batch_20260826fc_runtime198_capacity_preserves_material_descriptors() {
    let descriptors = material_editor_foundation_descriptors();
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    assert!(!descriptors.is_empty());
    assert!(descriptors.capacity() >= descriptors.len());
    assert_eq!(descriptors.len(), registry.len());
    assert_eq!(MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT, 25);
    assert!(
        descriptors
            .iter()
            .all(|descriptor| registry.contains(&descriptor.id))
    );
}

#[test]
fn optimization_batch_20260826fc_runtime198_material_catalog_reserves_group_total() {
    let source = include_str!("../mod.rs");
    assert!(source.contains("const MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT: usize = 25;"));
    assert!(source.contains("let descriptor_capacity = descriptor_groups"));
    assert!(source.contains("Vec::with_capacity(descriptor_capacity)"));
    assert!(source.contains("for group in descriptor_groups"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fc_runtime198_material_foundation_descriptor_capacity_bench() {
    assert_eq!(GROUP_SIZES.iter().sum::<usize>(), DESCRIPTORS_PER_BUILD);
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
        "RUNTIME198_MATERIAL_FOUNDATION_DESCRIPTOR_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} descriptor_groups={MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT} \
descriptors_per_build={DESCRIPTORS_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut descriptors = if reserve {
            Vec::with_capacity(DESCRIPTORS_PER_BUILD)
        } else {
            Vec::new()
        };
        for group_size in GROUP_SIZES {
            descriptors.extend((0..group_size).map(black_box));
        }
        checksum ^= black_box(descriptors.len() ^ descriptors.capacity());
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
