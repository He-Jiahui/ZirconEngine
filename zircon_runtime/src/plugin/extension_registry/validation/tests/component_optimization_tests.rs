use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::framework::scene::ComponentPropertyDescriptor;

use super::*;

const PROPERTY_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_PROPERTY_COUNT: usize = 8_192;
const PREALLOCATED_PROPERTY_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn runtime42_hash_batch_component_p95_uses_nearest_rank() {
    let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();

    assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
}

fn property_names() -> Vec<String> {
    (0..PROPERTY_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "generated_component_property_with_long_identity_{:05}",
                (index * 4_099) % UNIQUE_PROPERTY_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(property_names: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    property_names
        .iter()
        .filter(|name| unique.insert(name.as_str()))
        .count()
}

fn hash_unique_count(property_names: &[String]) -> usize {
    let mut unique = HashSet::new();
    property_names
        .iter()
        .filter(|name| unique.insert(name.as_str()))
        .count()
}

fn preallocation_property_names() -> Vec<String> {
    (0..PREALLOCATED_PROPERTY_COUNT)
        .map(|index| format!("generated_component_property_preallocated_{index:05}"))
        .collect()
}

fn unreserved_hash_unique_count(property_names: &[String]) -> usize {
    let mut unique = HashSet::new();
    property_names
        .iter()
        .filter(|name| unique.insert(name.as_str()))
        .count()
}

fn reserved_hash_unique_count(property_names: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(property_names.len());
    property_names
        .iter()
        .filter(|name| unique.insert(name.as_str()))
        .count()
}

#[test]
fn runtime42_hash_batch_component_preserves_first_duplicate_error() {
    let descriptor = ComponentTypeDescriptor {
        type_id: "sample.component".to_string(),
        plugin_id: "sample".to_string(),
        display_name: "Sample Component".to_string(),
        properties: vec![
            ComponentPropertyDescriptor {
                name: "enabled".to_string(),
                value_type: "bool".to_string(),
                editable: true,
            },
            ComponentPropertyDescriptor {
                name: "weight".to_string(),
                value_type: "f32".to_string(),
                editable: true,
            },
            ComponentPropertyDescriptor {
                name: "enabled".to_string(),
                value_type: "bool".to_string(),
                editable: false,
            },
        ],
    };

    let error = validate_component_type_descriptor(&descriptor).unwrap_err();
    assert!(format!("{error:?}").contains("property `enabled` must be unique"));
}

#[test]
fn runtime42_hash_batch_component_uses_borrowed_hash_set() {
    let source = include_str!("../component.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(production
        .contains("let mut property_names = HashSet::with_capacity(descriptor.properties.len());"));
    assert!(production.contains("property_names.insert(property.name.as_str())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime42_hash_batch_component_property_performance_evidence() {
    let property_names = property_names();
    assert_eq!(
        ordered_unique_count(&property_names),
        hash_unique_count(&property_names)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&property_names)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&property_names)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&property_names)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&property_names)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME42_COMPONENT_PROPERTY_HASH_VALIDATION_BENCH_V1 \
             admissions={PROPERTY_ADMISSION_COUNT} unique_properties={UNIQUE_PROPERTY_COUNT} \
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
fn optimization_batch_ii_runtime619_component_property_validation_preallocates_hash_storage() {
    let source = include_str!("../component.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production
        .contains("let mut property_names = HashSet::with_capacity(descriptor.properties.len());"));
    assert!(production.contains("property_names.insert(property.name.as_str())"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ii_runtime619_preallocated_component_property_performance_evidence() {
    let property_names = preallocation_property_names();
    assert_eq!(
        unreserved_hash_unique_count(&property_names),
        reserved_hash_unique_count(&property_names)
    );

    black_box(unreserved_hash_unique_count(black_box(&property_names)));
    black_box(reserved_hash_unique_count(black_box(&property_names)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&property_names)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&property_names)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&property_names)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&property_names)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "RUNTIME619_PREALLOCATED_COMPONENT_PROPERTY_BENCH_V1 \
             properties={PREALLOCATED_PROPERTY_COUNT} borrowed_identity=true \
             unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated property validation P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}
