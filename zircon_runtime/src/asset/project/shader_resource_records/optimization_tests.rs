use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};

use super::deduplicate_shader_resource_records;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 64;
const RECORD_COUNT: usize = 4_096;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_records() -> Vec<ResourceRecord> {
    (0..RECORD_COUNT)
        .rev()
        .map(|index| {
            let locator =
                ResourceLocator::parse(&format!("res://shader/record-{index:05}.zshader"))
                    .expect("valid shader resource fixture locator");
            ResourceRecord::new(
                ResourceId::from_locator(&locator),
                ResourceKind::Shader,
                locator,
            )
        })
        .collect()
}

fn legacy_deduplicate_shader_resource_records(records: Vec<ResourceRecord>) -> Vec<ResourceRecord> {
    let mut records_by_id = BTreeMap::new();
    for record in records {
        records_by_id.insert(record.id, record);
    }
    let mut records = records_by_id.into_values().collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.primary_locator
            .cmp(&right.primary_locator)
            .then_with(|| left.id.cmp(&right.id))
    });
    records
}

#[test]
fn runtime04_registry_records_shader_resource_sort_preserves_canonical_locator_order() {
    let legacy = legacy_deduplicate_shader_resource_records(fixture_records());
    let optimized = deduplicate_shader_resource_records(fixture_records())
        .expect("fixture records have unique IDs and locators");

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.len(), RECORD_COUNT);
    assert!(optimized.windows(2).all(|window| {
        window[0].primary_locator <= window[1].primary_locator
            && (window[0].primary_locator < window[1].primary_locator
                || window[0].id <= window[1].id)
    }));
}

#[test]
fn runtime04_registry_records_shader_resource_sort_source_contract() {
    let source = include_str!("../shader_resource_records.rs");
    assert!(source.contains("records.sort_unstable_by(|left, right|"));
    assert!(!source.contains("records.sort_by(|left, right|"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_registry_records_shader_resource_sort_bench() {
    let fixture = fixture_records();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_deduplicate_shader_resource_records(fixture.clone()));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(deduplicate_shader_resource_records(fixture.clone()));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME04_SHADER_RESOURCE_RECORD_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} records={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        RECORD_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
