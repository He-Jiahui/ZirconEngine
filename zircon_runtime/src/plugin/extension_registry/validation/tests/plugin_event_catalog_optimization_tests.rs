use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::plugin::PluginEventManifest;

use super::*;

const EVENT_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_EVENT_COUNT: usize = 8_192;
const PREALLOCATED_EVENT_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn runtime42_hash_batch_plugin_event_p95_uses_nearest_rank() {
    let mut samples = (1..=17).map(Duration::from_nanos).collect::<Vec<_>>();

    assert_eq!(percentile_95(&mut samples), Duration::from_nanos(17));
}

fn event_ids() -> Vec<String> {
    (0..EVENT_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "sample.events.generated_event_with_long_identity_{:05}",
                (index * 4_099) % UNIQUE_EVENT_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(event_ids: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    event_ids
        .iter()
        .filter(|event_id| unique.insert(event_id.as_str()))
        .count()
}

fn hash_unique_count(event_ids: &[String]) -> usize {
    let mut unique = HashSet::new();
    event_ids
        .iter()
        .filter(|event_id| unique.insert(event_id.as_str()))
        .count()
}

fn preallocation_event_ids() -> Vec<String> {
    (0..PREALLOCATED_EVENT_COUNT)
        .map(|index| format!("sample.events.preallocated_event_identity_{index:05}"))
        .collect()
}

fn unreserved_hash_unique_count(event_ids: &[String]) -> usize {
    let mut unique = HashSet::new();
    event_ids
        .iter()
        .filter(|event_id| unique.insert(event_id.as_str()))
        .count()
}

fn reserved_hash_unique_count(event_ids: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(event_ids.len());
    event_ids
        .iter()
        .filter(|event_id| unique.insert(event_id.as_str()))
        .count()
}

fn event(id: &str, display_name: &str) -> PluginEventManifest {
    PluginEventManifest {
        id: id.to_string(),
        display_name: display_name.to_string(),
        payload_schema: String::new(),
    }
}

#[test]
fn runtime42_hash_batch_plugin_event_preserves_first_duplicate_error() {
    let descriptor = PluginEventCatalogManifest {
        namespace: "sample.events".to_string(),
        version: 1,
        events: vec![
            event("sample.events.open", "Open"),
            event("sample.events.close", "Close"),
            event("sample.events.open", "Duplicate Open"),
        ],
    };

    let error = validate_plugin_event_catalog_manifest(&descriptor).unwrap_err();
    assert!(format!("{error:?}")
        .contains("event id `sample.events.open` must be unique inside catalog `sample.events`"));
}

#[test]
fn runtime42_hash_batch_plugin_event_uses_borrowed_hash_set() {
    let source = include_str!("../plugin_event_catalog.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(
        production.contains("let mut event_ids = HashSet::with_capacity(descriptor.events.len());")
    );
    assert!(production.contains("event_ids.insert(event.id.as_str())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime42_hash_batch_plugin_event_performance_evidence() {
    let event_ids = event_ids();
    assert_eq!(
        ordered_unique_count(&event_ids),
        hash_unique_count(&event_ids)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&event_ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&event_ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&event_ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&event_ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME42_PLUGIN_EVENT_ID_HASH_VALIDATION_BENCH_V1 admissions={EVENT_ADMISSION_COUNT} \
             unique_events={UNIQUE_EVENT_COUNT} borrowed_identity=true \
             ordered_p95_ns={} hash_p95_ns={}",
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
fn optimization_batch_ij_runtime620_plugin_event_validation_preallocates_hash_storage() {
    let source = include_str!("../plugin_event_catalog.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(
        production.contains("let mut event_ids = HashSet::with_capacity(descriptor.events.len());")
    );
    assert!(production.contains("event_ids.insert(event.id.as_str())"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ij_runtime620_preallocated_plugin_event_performance_evidence() {
    let event_ids = preallocation_event_ids();
    assert_eq!(
        unreserved_hash_unique_count(&event_ids),
        reserved_hash_unique_count(&event_ids)
    );

    black_box(unreserved_hash_unique_count(black_box(&event_ids)));
    black_box(reserved_hash_unique_count(black_box(&event_ids)));

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&event_ids)));
            unreserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&event_ids)));
            reserved_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(reserved_hash_unique_count(black_box(&event_ids)));
            reserved_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(unreserved_hash_unique_count(black_box(&event_ids)));
            unreserved_samples.push(started.elapsed());
        }
    }

    let unreserved_p95 = percentile_95(&mut unreserved_samples);
    let reserved_p95 = percentile_95(&mut reserved_samples);
    println!(
        "RUNTIME620_PREALLOCATED_PLUGIN_EVENT_BENCH_V1 \
             events={PREALLOCATED_EVENT_COUNT} borrowed_identity=true \
             unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_p95.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos() * 100 <= unreserved_p95.as_nanos() * 85,
        "preallocated event validation P95 {:?} exceeded 85% of unreserved P95 {:?}",
        reserved_p95,
        unreserved_p95,
    );
}
