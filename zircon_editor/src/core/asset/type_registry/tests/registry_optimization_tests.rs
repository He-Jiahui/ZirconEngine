use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const PERFORMANCE_LABEL_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

fn legacy_unique_menu_label(
    label: String,
    used_labels: &mut BTreeSet<String>,
    next_suffix_by_base: &mut BTreeMap<String, usize>,
) -> String {
    if used_labels.insert(label.clone()) {
        return label;
    }
    let next_suffix = next_suffix_by_base.entry(label.clone()).or_insert(2);
    loop {
        let ordinal = *next_suffix;
        *next_suffix += 1;
        let candidate = format!("{label} {ordinal}");
        if used_labels.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn legacy_label_index(labels: &[String]) -> usize {
    let mut label_counts = BTreeMap::<String, usize>::new();
    for label in labels {
        *label_counts.entry(label.clone()).or_default() += 1;
    }
    let mut used_labels = BTreeSet::new();
    let mut next_suffix_by_base = BTreeMap::new();
    labels
        .iter()
        .map(|label| {
            let label = if label_counts.get(label).copied().unwrap_or_default() > 1 {
                format!("{label} (duplicate)")
            } else {
                label.clone()
            };
            legacy_unique_menu_label(label, &mut used_labels, &mut next_suffix_by_base).len()
        })
        .sum()
}

fn hash_label_index(labels: &[String]) -> usize {
    let mut label_counts = HashMap::<String, usize>::with_capacity(labels.len());
    for label in labels {
        *label_counts.entry(label.clone()).or_default() += 1;
    }
    let mut used_labels = HashSet::with_capacity(labels.len());
    let mut next_suffix_by_base = HashMap::new();
    labels
        .iter()
        .map(|label| {
            let label = if label_counts.get(label).copied().unwrap_or_default() > 1 {
                format!("{label} (duplicate)")
            } else {
                label.clone()
            };
            unique_menu_label(label, &mut used_labels, &mut next_suffix_by_base).len()
        })
        .sum()
}

fn projected_labels_with_trees(labels: &[&str]) -> Vec<String> {
    let mut used_labels = BTreeSet::new();
    let mut next_suffix_by_base = BTreeMap::new();
    labels
        .iter()
        .map(|label| {
            legacy_unique_menu_label(
                (*label).to_string(),
                &mut used_labels,
                &mut next_suffix_by_base,
            )
        })
        .collect()
}

fn projected_labels_with_hashes(labels: &[&str]) -> Vec<String> {
    let mut used_labels = HashSet::new();
    let mut next_suffix_by_base = HashMap::new();
    labels
        .iter()
        .map(|label| {
            unique_menu_label(
                (*label).to_string(),
                &mut used_labels,
                &mut next_suffix_by_base,
            )
        })
        .collect()
}

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826e_editor57_creation_menu_uses_hash_membership_indexes() {
    let source = include_str!("../registry.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("let mut label_counts = HashMap::<String, usize>::with_capacity"));
    assert!(source.contains("let mut used_labels = HashSet::with_capacity"));
    assert!(source.contains("let mut next_suffix_by_base = HashMap::new()"));
    assert!(source.contains("used_labels: &mut HashSet<String>"));
    assert!(source.contains("next_suffix_by_base: &mut HashMap<String, usize>"));
}

#[test]
fn optimization_batch_20260826e_editor57_creation_menu_hash_indexes_preserve_labels() {
    let labels = [
        "Create Material",
        "Create Material",
        "Create Material 2",
        "Create Material",
        "Create Material 2",
        "Create Texture",
        "Create Texture",
    ];
    assert_eq!(
        projected_labels_with_hashes(&labels),
        projected_labels_with_trees(&labels)
    );
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826e_editor57_creation_menu_hash_index_performance_evidence() {
    let labels = (0..PERFORMANCE_LABEL_COUNT)
        .map(|index| format!("Create Asset Type {index:05}"))
        .collect::<Vec<_>>();
    assert_eq!(legacy_label_index(&labels), hash_label_index(&labels));

    for _ in 0..3 {
        black_box(legacy_label_index(black_box(&labels)));
        black_box(hash_label_index(black_box(&labels)));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_micros(|| {
                black_box(legacy_label_index(black_box(&labels)));
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                black_box(hash_label_index(black_box(&labels)));
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "EDITOR57_CREATION_MENU_HASH_INDEX_BENCH_V1 sample_pairs={} labels={} legacy_index=balanced_tree optimized_index=hash legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        SAMPLE_PAIRS,
        PERFORMANCE_LABEL_COUNT,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "hash-only membership index p95 must be at least 30% below balanced-tree indexes: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}
