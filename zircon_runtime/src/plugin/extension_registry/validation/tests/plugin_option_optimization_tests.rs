use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ENUM_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_ENUM_COUNT: usize = 8_192;
const PREALLOCATED_ENUM_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn runtime42_hash_batch_plugin_option_p95_uses_nearest_rank() {
    let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();

    assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
}

fn enum_values() -> Vec<String> {
    (0..ENUM_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "generated_enum_value_with_long_shared_identity_{:05}",
                (index * 4_099) % UNIQUE_ENUM_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(enum_values: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    enum_values
        .iter()
        .filter(|value| unique.insert(value.as_str()))
        .count()
}

fn hash_unique_count(enum_values: &[String]) -> usize {
    let mut unique = HashSet::new();
    enum_values
        .iter()
        .filter(|value| unique.insert(value.as_str()))
        .count()
}

fn preallocation_enum_values() -> Vec<String> {
    (0..PREALLOCATED_ENUM_COUNT)
        .map(|index| format!("generated_enum_value_preallocated_{index:05}"))
        .collect()
}

fn unreserved_hash_unique_count(enum_values: &[String]) -> usize {
    let mut unique = HashSet::new();
    enum_values
        .iter()
        .filter(|value| unique.insert(value.as_str()))
        .count()
}

fn reserved_hash_unique_count(enum_values: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(enum_values.len());
    enum_values
        .iter()
        .filter(|value| unique.insert(value.as_str()))
        .count()
}

#[test]
fn runtime42_hash_batch_plugin_option_preserves_first_duplicate_error() {
    let descriptor =
        PluginOptionManifest::new("sample.render.mode", "Render Mode", "enum", "quality")
            .with_enum_values(["quality", "performance", "quality"]);

    let error = validate_plugin_option_manifest(&descriptor).unwrap_err();
    assert!(format!("{error:?}")
        .contains("sample.render.mode enum_values entry `quality` must be unique"));
}

#[test]
fn runtime42_hash_batch_plugin_option_uses_borrowed_hash_set() {
    let source = include_str!("../plugin_option.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(production
        .contains("let mut seen_values = HashSet::with_capacity(descriptor.enum_values.len());"));
    assert!(production.contains("seen_values.insert(enum_value)"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime42_hash_batch_plugin_option_performance_evidence() {
    let enum_values = enum_values();
    assert_eq!(
        ordered_unique_count(&enum_values),
        hash_unique_count(&enum_values)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&enum_values)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&enum_values)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&enum_values)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&enum_values)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME42_PLUGIN_OPTION_ENUM_HASH_VALIDATION_BENCH_V1 \
             admissions={ENUM_ADMISSION_COUNT} unique_values={UNIQUE_ENUM_COUNT} \
             borrowed_identity=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}

#[test]
fn optimization_batch_ik_runtime621_plugin_option_validation_preallocates_hash_storage() {
    let source = include_str!("../plugin_option.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production
        .contains("let mut seen_values = HashSet::with_capacity(descriptor.enum_values.len());"));
    assert!(production.contains("seen_values.insert(enum_value)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ik_runtime621_preallocated_plugin_option_performance_evidence() {
    let enum_values = preallocation_enum_values();
    assert_eq!(
        unreserved_hash_unique_count(&enum_values),
        reserved_hash_unique_count(&enum_values)
    );

    black_box(unreserved_hash_unique_count(black_box(&enum_values)));
    black_box(reserved_hash_unique_count(black_box(&enum_values)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&enum_values)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&enum_values)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&enum_values)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&enum_values)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "RUNTIME621_PREALLOCATED_PLUGIN_OPTION_BENCH_V1 \
             enum_values={PREALLOCATED_ENUM_COUNT} borrowed_identity=true \
             unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated enum validation P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}
