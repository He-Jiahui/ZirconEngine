use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_MARKER: &str = "EDITOR23_PALETTE_CONTROL_ID_HASH_INDEX_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4;
const DENSE_ID_COUNT: usize = 2_048;

fn legacy_unique_control_id(existing: &[String], label: &str) -> String {
    let base = label
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if !existing.iter().any(|control_id| control_id == &base) {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}{index}");
        if !existing.iter().any(|control_id| control_id == &candidate) {
            return candidate;
        }
    }
    unreachable!("loop should always return a unique control id")
}

fn optimized_unique_control_id(existing: &[String], label: &str) -> String {
    let existing_control_ids = existing.iter().map(String::as_str).collect::<HashSet<_>>();
    unique_control_id_from_existing(&existing_control_ids, label)
}

fn dense_control_ids() -> Vec<String> {
    let mut ids = Vec::with_capacity(DENSE_ID_COUNT);
    ids.push("Button".to_string());
    ids.extend((2..=DENSE_ID_COUNT).map(|index| format!("Button{index}")));
    ids
}

fn sample_ns(mut lookup: impl FnMut() -> String) -> u128 {
    let started = Instant::now();
    let mut observed = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        observed += black_box(lookup()).len();
    }
    black_box(observed);
    started.elapsed().as_nanos()
}

fn percentile(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826bb_palette_control_id_hash_index_preserves_smallest_gap() {
    let existing = ["Button", "Button2", "Button4"]
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(
        unique_control_id_from_existing(&existing, "Button"),
        "Button3"
    );
    assert_eq!(
        unique_control_id_from_existing(&existing, "Icon Button"),
        "IconButton"
    );
    assert_eq!(unique_control_id_from_existing(&existing, "---"), "");
}

#[test]
fn optimization_batch_20260826bb_palette_control_id_uses_one_hash_index() {
    let source = include_str!("../instantiate.rs");

    assert!(source.contains("collect::<HashSet<_>>()"));
    assert!(source.contains("existing_control_ids.contains("));
    assert!(source.contains("fn unique_control_id_from_existing("));
    assert!(!source.contains("iter_nodes()\n        .any"));
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260826bb_palette_control_id_hash_index_p95() {
    let existing = dense_control_ids();
    assert_eq!(
        legacy_unique_control_id(&existing, "Button"),
        optimized_unique_control_id(&existing, "Button")
    );
    for _ in 0..4 {
        black_box(legacy_unique_control_id(&existing, "Button"));
        black_box(optimized_unique_control_id(&existing, "Button"));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(|| legacy_unique_control_id(&existing, "Button")));
            optimized_samples.push(sample_ns(|| {
                optimized_unique_control_id(&existing, "Button")
            }));
        } else {
            optimized_samples.push(sample_ns(|| {
                optimized_unique_control_id(&existing, "Button")
            }));
            legacy_samples.push(sample_ns(|| legacy_unique_control_id(&existing, "Button")));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples.clone(), 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction = 100.0 - (optimized_p95 as f64 * 100.0 / legacy_p95 as f64);
    println!(
        "{BENCHMARK_MARKER} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} reduction_percent={reduction:.3} dense_ids={DENSE_ID_COUNT} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS}"
    );

    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(10),
        "expected hash-indexed control ID P95 to be at least 90% below repeated node scans; legacy={legacy_p95}ns optimized={optimized_p95}ns reduction={reduction:.3}%"
    );
}
