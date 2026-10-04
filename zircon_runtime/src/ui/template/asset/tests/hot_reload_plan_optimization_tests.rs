use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ASSET_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_ASSET_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 101;

fn nearest_rank(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn asset_ids() -> Vec<String> {
    (0..ASSET_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "res://ui/hot_reload/{:04}.zui",
                (index * 4_099) % UNIQUE_ASSET_COUNT
            )
        })
        .collect()
}

fn legacy_admission_count(asset_ids: &[String]) -> usize {
    let mut targets = Vec::new();
    let mut seen = BTreeSet::new();
    for asset_id in asset_ids {
        if seen.insert(asset_id.to_string()) {
            targets.push(asset_id.to_string());
        }
    }
    black_box(targets).len()
}

fn optimized_admission_count(asset_ids: &[String]) -> usize {
    let mut targets = Vec::new();
    let mut seen = HashSet::new();
    for asset_id in asset_ids {
        if seen.contains(asset_id.as_str()) {
            continue;
        }
        let asset_id = asset_id.to_string();
        seen.insert(asset_id.clone());
        targets.push(asset_id);
    }
    black_box(targets).len()
}

#[test]
fn runtime74_batch_hash_admission_preserves_first_seen_order() {
    let mut targets = Vec::new();
    let mut seen = HashSet::new();
    for asset in ["res://b.zui", "res://a.zui", "res://b.zui"] {
        push_unique(&mut targets, &mut seen, asset);
    }

    assert_eq!(targets, vec!["res://b.zui", "res://a.zui"]);
    assert_eq!(seen.len(), 2);
}

#[test]
fn runtime74_batch_hot_reload_uses_hash_admission_sets() {
    let source = include_str!("../hot_reload_plan.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert_eq!(production.matches("HashSet<String>").count(), 8);
    assert_eq!(production.matches("HashSet::new()").count(), 7);
    assert!(production.contains("collect::<HashSet<_>>()"));
    assert!(!production.contains("BTreeSet"));
    assert!(
        production.find("seen.contains(value)").unwrap()
            < production.find("let value = value.to_string();").unwrap()
    );
}

#[test]
#[ignore = "release performance evidence"]
fn runtime74_batch_hot_reload_hash_admission_performance_evidence() {
    let asset_ids = asset_ids();
    assert_eq!(legacy_admission_count(&asset_ids), UNIQUE_ASSET_COUNT);
    assert_eq!(optimized_admission_count(&asset_ids), UNIQUE_ASSET_COUNT);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_admission_count(black_box(&asset_ids)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(optimized_admission_count(black_box(&asset_ids)));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(optimized_admission_count(black_box(&asset_ids)));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_admission_count(black_box(&asset_ids)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p50 = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95 = nearest_rank(&mut legacy_samples, 95);
    let legacy_p99 = nearest_rank(&mut legacy_samples.clone(), 99);
    let optimized_p50 = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95 = nearest_rank(&mut optimized_samples, 95);
    let optimized_p99 = nearest_rank(&mut optimized_samples.clone(), 99);
    println!(
        "RUNTIME74_HOT_RELOAD_HASH_ADMISSION_BENCH_V1 admissions={ASSET_ADMISSION_COUNT} \
             unique_assets={UNIQUE_ASSET_COUNT} sample_pairs={SAMPLE_COUNT} \
             pair_order=alternating_legacy_even legacy_first_pairs=51 optimized_first_pairs=50 \
             legacy_string_allocations={} optimized_string_allocations={} \
             legacy_p50_ns={} legacy_p95_ns={} legacy_p99_ns={} \
             optimized_p50_ns={} optimized_p95_ns={} optimized_p99_ns={}",
        ASSET_ADMISSION_COUNT + UNIQUE_ASSET_COUNT,
        UNIQUE_ASSET_COUNT * 2,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        legacy_p99.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        optimized_p99.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
        "hash-admission P95 {:?} exceeded 60% of tree-admission P95 {:?}",
        optimized_p95,
        legacy_p95,
    );
}
