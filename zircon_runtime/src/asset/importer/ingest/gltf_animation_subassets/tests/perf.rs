use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const DEPENDENCY_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn dependencies() -> Vec<String> {
    (0..DEPENDENCY_COUNT)
        .map(|index| format!("res://generated/gltf/Node{index:05}"))
        .collect()
}

fn legacy_unique_count(dependencies: &[String]) -> usize {
    let mut index = HashSet::new();
    dependencies
        .iter()
        .filter(|dependency| index.insert(dependency.as_str()))
        .count()
}

fn optimized_unique_count(dependencies: &[String]) -> usize {
    let mut index = HashSet::with_capacity(dependencies.len());
    dependencies
        .iter()
        .filter(|dependency| index.insert(dependency.as_str()))
        .count()
}

#[test]
fn optimization_batch_ir_runtime628_gltf_skin_dependency_index_uses_iterator_bound() {
    let source = include_str!("../../gltf_animation_subassets.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("glTF animation import production source");

    assert!(production.contains("skin.joints().size_hint()"));
    assert!(production.contains("usize::from(skin.skeleton().is_some())"));
    assert!(production.contains("usize::from(matrices_uri.is_some())"));
    assert!(production.contains("HashSet::with_capacity(dependency_capacity)"));
    assert!(!production.contains("let mut dependency_index = HashSet::new()"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_ir_runtime628_gltf_skin_dependency_index_performance_evidence() {
    let dependencies = dependencies();
    assert_eq!(
        legacy_unique_count(&dependencies),
        optimized_unique_count(&dependencies)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_unique_count(black_box(&dependencies)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_unique_count(black_box(&dependencies)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_unique_count(black_box(&dependencies)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_unique_count(black_box(&dependencies)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "RUNTIME628_PREALLOCATED_GLTF_SKIN_DEPENDENCY_BENCH_V1 dependencies={DEPENDENCY_COUNT} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated glTF dependency P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
