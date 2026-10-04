use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

const PARAM_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn param_names() -> Vec<String> {
    (0..PARAM_COUNT)
        .map(|index| format!("post.generated.long_parameter_name_{index:05}"))
        .collect()
}

fn legacy_unique_count(names: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    names
        .iter()
        .filter(|name| unique.insert(name.as_str()))
        .count()
}

fn optimized_unique_count(names: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(names.len());
    names
        .iter()
        .filter(|name| unique.insert(name.as_str()))
        .count()
}

#[test]
fn optimization_batch_in_runtime624_volume_param_validation_preallocates_hash_membership() {
    let source = include_str!("../../volume_registry.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("volume registry production source");

    assert!(production.contains("use std::collections::{HashMap, HashSet};"));
    assert!(production.contains("HashSet::with_capacity(descriptor.params.len())"));
    assert!(production.contains("param_names.insert(param.name)"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_in_runtime624_volume_param_hash_membership_performance_evidence() {
    let names = param_names();
    assert_eq!(legacy_unique_count(&names), optimized_unique_count(&names));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_unique_count(black_box(&names)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_unique_count(black_box(&names)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_unique_count(black_box(&names)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_unique_count(black_box(&names)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "RUNTIME624_PREALLOCATED_VOLUME_PARAM_BENCH_V1 params={PARAM_COUNT} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=4000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(4_000),
        "preallocated hash P95 {optimized_p95} ns exceeded 40% of tree P95 {legacy_p95} ns"
    );
}
