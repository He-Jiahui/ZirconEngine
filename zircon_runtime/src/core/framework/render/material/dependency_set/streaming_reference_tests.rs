use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::resource::ResourceLocator;

const REFERENCE_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hj_runtime256_preserves_material_reference_order() {
    let shader = reference("res://shaders/material.wgsl");
    let albedo = reference("res://textures/albedo.png");
    let normal = reference("res://textures/normal.png");
    let mut dependencies = RenderMaterialDependencySet::new(shader.clone());
    dependencies.push_texture(albedo.clone());
    dependencies.push_texture(normal.clone());
    dependencies.push_texture(albedo.clone());

    assert_eq!(dependencies.all_references(), vec![shader, albedo, normal]);
}

#[test]
fn optimization_batch_20260826hj_runtime256_streams_material_reference_clones() {
    let source = include_str!("../dependency_set.rs");
    let start = source
        .find("pub fn all_references(")
        .expect("all_references function");
    let end = source[start..]
        .find("\n    }")
        .map(|offset| start + offset)
        .expect("all_references boundary");
    let body = &source[start..end];

    assert!(body.contains("extend_cloned_values(&mut references, &self.textures)"));
    assert!(!body.contains("self.textures.clone()"));
    assert!(source.contains("target.extend_from_slice(source)"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hj_runtime256_streaming_material_reference_release_benchmark() {
    let source = (0..REFERENCE_COUNT)
        .map(|value| value as u64)
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(REFERENCE_COUNT);
    let mut optimized = Vec::with_capacity(REFERENCE_COUNT);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                legacy.clear();
                legacy_extend_cloned_values(black_box(&mut legacy), black_box(&source));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                optimized.clear();
                extend_cloned_values(black_box(&mut optimized), black_box(&source));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME256_STREAMING_MATERIAL_REFERENCE_BENCH_V1 \
         reference_count={REFERENCE_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(ResourceLocator::parse(locator).expect("valid resource locator"))
}

fn legacy_extend_cloned_values<T: Clone>(target: &mut Vec<T>, source: &[T]) {
    target.extend(source.to_vec());
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
