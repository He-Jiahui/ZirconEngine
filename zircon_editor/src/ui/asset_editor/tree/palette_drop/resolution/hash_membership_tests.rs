use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const SLOT_COUNT: usize = 2_048;
const SAMPLE_COUNT: usize = 17;
const SLOT_GROUPS: &[&[&str]] = &[
    &["header", "top"],
    &["content", "body", "center", "main", "default"],
    &["footer", "bottom"],
];

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn slots() -> Vec<String> {
    (0..SLOT_COUNT)
        .map(|index| format!("header_generated_slot_with_long_identity_{index:05}"))
        .collect()
}

fn legacy_ordered_component_slot_names(available: &[String], groups: &[&[&str]]) -> Vec<String> {
    let mut ordered = Vec::new();
    for semantics in groups {
        for slot_name in available {
            if ordered.iter().any(|existing| existing == slot_name) {
                continue;
            }
            if semantics
                .iter()
                .any(|semantic| normalized_slot_name(slot_name).contains(semantic))
            {
                ordered.push(slot_name.clone());
            }
        }
    }
    for slot_name in available {
        if !ordered.iter().any(|existing| existing == slot_name) {
            ordered.push(slot_name.clone());
        }
    }
    ordered
}

#[test]
fn optimization_batch_20260826ag_editor23_hash_slot_admission_preserves_semantic_and_source_order()
{
    let available = vec![
        "footer_slot".to_string(),
        "content_slot".to_string(),
        "header_slot".to_string(),
        "content_slot".to_string(),
        "unclassified_slot".to_string(),
    ];

    assert_eq!(
        ordered_component_slot_names(&available, SLOT_GROUPS),
        vec![
            "header_slot",
            "content_slot",
            "footer_slot",
            "unclassified_slot",
        ]
    );
}

#[test]
fn optimization_batch_20260826ag_editor23_slot_order_uses_hash_admission_and_vector_publication() {
    let source = include_str!("../resolution.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let ordering = production
        .split("fn ordered_component_slot_names")
        .nth(1)
        .and_then(|body| body.split("fn component_mount_for_node").next())
        .expect("slot ordering implementation");

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(ordering.contains("HashSet::with_capacity(available.len())"));
    assert!(ordering.contains("Vec::with_capacity(available.len())"));
    assert!(ordering.contains("admitted.insert(slot_name.as_str())"));
    assert!(!ordering.contains("ordered.iter().any"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ag_editor23_palette_slot_hash_admission_performance_evidence() {
    let slots = slots();
    assert_eq!(
        legacy_ordered_component_slot_names(&slots, SLOT_GROUPS),
        ordered_component_slot_names(&slots, SLOT_GROUPS)
    );

    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_ordered_component_slot_names(
                black_box(&slots),
                SLOT_GROUPS,
            ));
            linear_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_component_slot_names(black_box(&slots), SLOT_GROUPS));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(ordered_component_slot_names(black_box(&slots), SLOT_GROUPS));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_ordered_component_slot_names(
                black_box(&slots),
                SLOT_GROUPS,
            ));
            linear_samples.push(started.elapsed());
        }
    }

    let linear_p95 = percentile_95(&mut linear_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR23_PALETTE_SLOT_HASH_ADMISSION_BENCH_V1 \
         slots={SLOT_COUNT} semantic_groups={} stable_vector_order=true \
         linear_p95_ns={} hash_p95_ns={}",
        SLOT_GROUPS.len(),
        linear_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= linear_p95.as_nanos() * 60,
        "hash-admission P95 {:?} exceeded 60% of linear-admission P95 {:?}",
        hash_p95,
        linear_p95,
    );
}
