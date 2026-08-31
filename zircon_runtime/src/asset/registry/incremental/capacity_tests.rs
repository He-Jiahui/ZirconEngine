use std::hint::black_box;
use std::time::Instant;

use super::watch_change_path_capacity;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::AssetUri;

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 64;
const CHANGES_PER_BATCH: usize = 4_096;

#[test]
fn optimization_batch_20260826ft_runtime215_capacity_covers_watcher_change_paths() {
    let changes = (0..CHANGES_PER_BATCH)
        .map(|index| AssetChange {
            kind: AssetChangeKind::Modified,
            uri: AssetUri::parse(&format!("res://textures/watch-{index}.png")).expect("asset URI"),
            previous_uri: None,
        })
        .collect::<Vec<_>>();
    let mut changed_paths = Vec::with_capacity(watch_change_path_capacity(&changes));
    changed_paths.extend(changes.iter().map(|change| change.uri.clone()));

    assert_eq!(changed_paths.len(), changes.len());
    assert!(changed_paths.capacity() >= changes.len());
    assert_eq!(
        changed_paths.first(),
        changes.first().map(|change| &change.uri)
    );
    assert_eq!(
        changed_paths.last(),
        changes.last().map(|change| &change.uri)
    );
}

#[test]
fn optimization_batch_20260826ft_runtime215_incremental_registry_reserves_change_count() {
    let source = include_str!("../incremental.rs");
    assert!(source.contains("Vec::with_capacity(watch_change_path_capacity(changes))"));
    assert!(source.contains("fn watch_change_path_capacity(changes: &[AssetChange]) -> usize"));
    assert!(source.contains("changes.len()"));
    assert!(!source.contains("let mut changed_paths = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ft_runtime215_watch_change_path_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME215_WATCH_CHANGE_PATH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} changes_per_batch={CHANGES_PER_BATCH} \
legacy_reservations_per_batch=0 optimized_reservations_per_batch=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct ChangePathFixture([usize; 4]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for batch in 0..BATCHES_PER_SAMPLE {
        let mut changed_paths = if reserve {
            Vec::with_capacity(CHANGES_PER_BATCH)
        } else {
            Vec::new()
        };
        for change in 0..CHANGES_PER_BATCH {
            changed_paths.push(ChangePathFixture([black_box(batch ^ change); 4]));
        }
        checksum ^= black_box(
            changed_paths.len()
                ^ changed_paths.capacity()
                ^ changed_paths[CHANGES_PER_BATCH - 1].0[0],
        );
        black_box(&changed_paths);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
