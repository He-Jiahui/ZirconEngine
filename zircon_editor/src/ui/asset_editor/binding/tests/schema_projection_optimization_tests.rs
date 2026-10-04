use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const PAYLOAD_KEY_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_PAYLOAD_KEY_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn payload_keys() -> Vec<String> {
    (0..PAYLOAD_KEY_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "payload.key.{:04}",
                (index * 4_099) % UNIQUE_PAYLOAD_KEY_COUNT
            )
        })
        .collect()
}

fn ordered_payload_key_count(payload_keys: &[String]) -> usize {
    let mut projected_payload_keys = BTreeSet::new();
    payload_keys
        .iter()
        .filter(|key| projected_payload_keys.insert((*key).clone()))
        .count()
}

fn hash_payload_key_count(payload_keys: &[String]) -> usize {
    let mut projected_payload_keys = HashSet::new();
    payload_keys
        .iter()
        .filter(|key| projected_payload_keys.insert((*key).clone()))
        .count()
}

fn preallocated_hash_payload_key_count(payload_keys: &[String]) -> usize {
    let mut projected_payload_keys = HashSet::with_capacity(payload_keys.len());
    payload_keys
        .iter()
        .filter(|key| projected_payload_keys.insert((*key).clone()))
        .count()
}

#[test]
fn optimization_batch_20260826u_editor23_hash_payload_membership_matches_ordered_membership() {
    let payload_keys = payload_keys();

    assert_eq!(
        ordered_payload_key_count(&payload_keys),
        hash_payload_key_count(&payload_keys)
    );
    assert_eq!(
        hash_payload_key_count(&payload_keys),
        UNIQUE_PAYLOAD_KEY_COUNT
    );
}

#[test]
fn optimization_batch_20260826u_editor23_payload_projection_uses_hash_membership() {
    let source = include_str!("../schema_projection.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains(
        "let mut projected_payload_keys = HashSet::with_capacity(payload_projection_capacity);"
    ));
    assert!(production.contains("let payload_projection_capacity ="));
    assert!(production.contains("BTreeMap<String, Vec<UiBindingDiagnostic>>"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
fn optimization_batch_r6_wave2_editor635_preallocated_payload_membership_is_equivalent() {
    let payload_keys = (0..PAYLOAD_KEY_ADMISSION_COUNT)
        .map(|index| format!("payload.unique.{index:05}"))
        .collect::<Vec<_>>();

    assert_eq!(
        hash_payload_key_count(&payload_keys),
        preallocated_hash_payload_key_count(&payload_keys)
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave2_editor635_preallocated_payload_membership_p95() {
    let payload_keys = (0..PAYLOAD_KEY_ADMISSION_COUNT)
        .map(|index| format!("payload.unique.{index:05}"))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(hash_payload_key_count(black_box(&payload_keys)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(preallocated_hash_payload_key_count(black_box(
                &payload_keys,
            )));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(preallocated_hash_payload_key_count(black_box(
                &payload_keys,
            )));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_payload_key_count(black_box(&payload_keys)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR635_PREALLOCATED_BINDING_PAYLOAD_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_COUNT} payload_keys={PAYLOAD_KEY_ADMISSION_COUNT} legacy_p95_ns={} optimized_p95_ns={} ratio={:.4}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        optimized_p95.as_nanos() as f64 / legacy_p95.as_nanos().max(1) as f64
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 85,
        "preallocated payload membership P95 {:?} exceeded 85% of unreserved {:?}",
        optimized_p95,
        legacy_p95
    );
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826u_editor23_payload_key_hash_membership_performance_evidence() {
    let payload_keys = payload_keys();
    assert_eq!(
        ordered_payload_key_count(&payload_keys),
        hash_payload_key_count(&payload_keys)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_payload_key_count(black_box(&payload_keys)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_payload_key_count(black_box(&payload_keys)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_payload_key_count(black_box(&payload_keys)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_payload_key_count(black_box(&payload_keys)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR23_PAYLOAD_KEY_HASH_MEMBERSHIP_BENCH_V1 admissions={PAYLOAD_KEY_ADMISSION_COUNT} \
             unique_payload_keys={UNIQUE_PAYLOAD_KEY_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
