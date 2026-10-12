use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

const NAME_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn schema_names() -> Vec<String> {
    (0..NAME_COUNT)
        .map(|index| format!("material.generated.long_schema_name_{index:05}"))
        .collect()
}

fn legacy_membership_count(names: &[String]) -> usize {
    let mut seen = BTreeSet::new();
    names
        .iter()
        .filter(|name| seen.insert(name.as_str()))
        .count()
}

fn optimized_membership_count(names: &[String]) -> usize {
    let mut seen = HashSet::with_capacity(names.len());
    names
        .iter()
        .filter(|name| seen.insert(name.as_str()))
        .count()
}

#[test]
fn optimization_batch_in_editor624_material_projection_preallocates_hash_membership() {
    let source = include_str!("../../projection.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("material projection production source");

    assert!(production.contains("use std::collections::HashSet;"));
    assert_eq!(production.matches("HashSet::with_capacity").count(), 2);
    assert!(production.contains("shader.property_schema.len()"));
    assert!(production.contains("shader.texture_slots.len()"));
    assert!(production.contains("seen.insert(property.name.as_str())"));
    assert!(production.contains("seen.insert(slot.name.as_str())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_in_editor624_material_projection_hash_membership_performance_evidence() {
    let names = schema_names();
    assert_eq!(
        legacy_membership_count(&names),
        optimized_membership_count(&names)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_membership_count(black_box(&names)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_membership_count(black_box(&names)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_membership_count(black_box(&names)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_membership_count(black_box(&names)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR624_PREALLOCATED_MATERIAL_PROJECTION_BENCH_V1 names={NAME_COUNT} \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=4000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(4_000),
        "preallocated hash P95 {optimized_p95} ns exceeded 40% of tree P95 {legacy_p95} ns"
    );
}
