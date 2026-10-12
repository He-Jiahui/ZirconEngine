use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};

use super::deduplicate_shader_resource_records;

const RECORD_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jc_runtime642_hashes_shader_resource_identity_indexes() {
    let source = include_str!("../../shader_resource_records.rs");
    let deduplication = source
        .split("fn deduplicate_shader_resource_records")
        .nth(1)
        .expect("shader resource deduplication remains present")
        .split("fn collect_shader_resource_records")
        .next()
        .expect("shader resource deduplication remains bounded");

    assert!(deduplication.contains("let record_capacity = records.len();"));
    assert_eq!(
        deduplication
            .matches("HashMap::with_capacity(record_capacity)")
            .count(),
        2
    );
    assert!(!deduplication.contains("BTreeMap<ResourceId, ResourceRecord>"));
    assert!(!deduplication.contains("BTreeMap<ResourceLocator, ResourceId>"));
    assert!(deduplication.contains("records.sort_unstable_by(|left, right|"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jc_runtime642_hashed_shader_resource_dedup_benchmark() {
    let fixture = fixture_records();
    for _ in 0..4 {
        black_box(measure_deduplication(&fixture, false));
        black_box(measure_deduplication(&fixture, true));
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hashed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_deduplication(&fixture, false));
            hashed_samples.push(measure_deduplication(&fixture, true));
        } else {
            hashed_samples.push(measure_deduplication(&fixture, true));
            ordered_samples.push(measure_deduplication(&fixture, false));
        }
    }

    let ordered_p95 = percentile(&ordered_samples, 95);
    let hashed_p95 = percentile(&hashed_samples, 95);
    let improvement_percent =
        ordered_p95.saturating_sub(hashed_p95).saturating_mul(100) / ordered_p95.max(1);
    println!(
        "RUNTIME642_HASHED_SHADER_RESOURCE_DEDUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} record_count={RECORD_COUNT} ordered_ns={} hashed_ns={} ordered_p95_ns={ordered_p95} hashed_p95_ns={hashed_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&ordered_samples),
        csv(&hashed_samples),
    );
    assert!(hashed_p95 <= ordered_p95 * 80 / 100);
}

fn fixture_records() -> Vec<ResourceRecord> {
    (0..RECORD_COUNT)
        .rev()
        .map(|index| {
            let locator =
                ResourceLocator::parse(&format!("res://shader/batch-642-{index:08}.zshader"))
                    .expect("valid synthetic shader locator");
            ResourceRecord::new(
                ResourceId::from_locator(&locator),
                ResourceKind::Shader,
                locator,
            )
        })
        .collect()
}

fn measure_deduplication(fixture: &[ResourceRecord], hashed: bool) -> u128 {
    let records = fixture.to_vec();
    let started = Instant::now();
    let records = if hashed {
        deduplicate_shader_resource_records(records)
            .expect("synthetic records have unique IDs and locators")
    } else {
        ordered_deduplicate(records)
    };
    black_box(records);
    started.elapsed().as_nanos().max(1)
}

fn ordered_deduplicate(records: Vec<ResourceRecord>) -> Vec<ResourceRecord> {
    let mut records_by_id = BTreeMap::new();
    let mut ids_by_locator = BTreeMap::new();
    for record in records {
        ids_by_locator.insert(record.primary_locator.clone(), record.id);
        records_by_id.insert(record.id, record);
    }
    let mut records = records_by_id.into_values().collect::<Vec<_>>();
    records.sort_unstable_by(|left, right| {
        left.primary_locator
            .cmp(&right.primary_locator)
            .then_with(|| left.id.cmp(&right.id))
    });
    records
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
