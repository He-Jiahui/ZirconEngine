use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const DEPENDENCY_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_DEPENDENCY_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn dependency_keys() -> Vec<(String, String)> {
    (0..DEPENDENCY_ADMISSION_COUNT)
        .map(|index| {
            let identity = (index * 4_099) % UNIQUE_DEPENDENCY_COUNT;
            (
                format!("generated.preview.node.with.long.identity.{identity:05}"),
                format!("binding.payload.deeply.nested.generated_value_{identity:05}"),
            )
        })
        .collect()
}

fn ordered_unique_count(keys: &[(String, String)]) -> usize {
    let mut unique = BTreeSet::new();
    keys.iter()
        .filter(|(node_id, path)| unique.insert((node_id.as_str(), path.as_str())))
        .count()
}

fn hash_unique_count(keys: &[(String, String)]) -> usize {
    let mut unique = HashSet::new();
    keys.iter()
        .filter(|(node_id, path)| unique.insert((node_id.as_str(), path.as_str())))
        .count()
}

#[test]
fn optimization_batch_20260826ab_editor23_hash_dependency_dedup_preserves_first_value_and_order() {
    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();
    push_dependency(
        &mut dependencies,
        &mut seen,
        "node.beta".to_string(),
        "binding.value".to_string(),
        Value::String("first".to_string()),
    );
    push_dependency(
        &mut dependencies,
        &mut seen,
        "node.alpha".to_string(),
        "binding.value".to_string(),
        Value::String("second".to_string()),
    );
    push_dependency(
        &mut dependencies,
        &mut seen,
        "node.beta".to_string(),
        "binding.value".to_string(),
        Value::String("duplicate".to_string()),
    );

    assert_eq!(
        dependencies,
        vec![
            (
                "node.beta".to_string(),
                "binding.value".to_string(),
                Value::String("first".to_string()),
            ),
            (
                "node.alpha".to_string(),
                "binding.value".to_string(),
                Value::String("second".to_string()),
            ),
        ]
    );
}

#[test]
fn optimization_batch_20260826ab_editor23_preview_dependency_dedup_uses_hash_membership() {
    let source = include_str!("../mock_value_resolution.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(production.contains("let mut seen = HashSet::new();"));
    assert!(production.contains("seen: &mut HashSet<(String, String)>"));
    assert!(production.contains("dependencies.push((target_node_id, target_path, target_value))"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ab_editor23_preview_dependency_hash_dedup_performance_evidence() {
    let keys = dependency_keys();
    assert_eq!(ordered_unique_count(&keys), hash_unique_count(&keys));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&keys)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&keys)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&keys)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&keys)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR23_PREVIEW_DEPENDENCY_HASH_DEDUP_BENCH_V1 \
         admissions={DEPENDENCY_ADMISSION_COUNT} unique_dependencies={UNIQUE_DEPENDENCY_COUNT} \
         first_occurrence_order=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-dedup P95 {:?} exceeded 60% of ordered-dedup P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
