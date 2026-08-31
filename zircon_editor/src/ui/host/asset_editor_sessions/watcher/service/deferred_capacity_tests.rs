use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::{collect_deferred_paths, UiAssetWorkspaceWatcher};
use crate::ui::host::asset_editor_sessions::watcher::budget::UiAssetWatchBudget;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 4_096;
const DEFERRED_PATHS_PER_BUILD: usize = 64;

#[test]
fn optimization_batch_20260826fo_editor156_capacity_preserves_deferred_path_order() {
    let budget = UiAssetWatchBudget::try_new(128, 128, Duration::from_secs(1))
        .expect("valid watcher budget");
    let watcher = UiAssetWorkspaceWatcher::without_notify_for_test(Vec::new(), budget);
    let expected = (0..128)
        .map(|index| PathBuf::from(format!("assets/material-{index}.zmat")))
        .collect::<Vec<_>>();
    watcher.record_paths_for_test(expected.clone());
    let drained = watcher.ingress.drain_paths(expected.len());
    let mut remaining = drained.into_iter();
    let first = remaining.next().expect("deferred path");

    let deferred = collect_deferred_paths(first, remaining);

    assert_eq!(deferred.len(), expected.len());
    assert!(deferred.capacity() >= expected.len());
    assert_eq!(
        deferred
            .iter()
            .map(|pending| pending.path.clone())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn optimization_batch_20260826fo_editor156_deferred_paths_reserve_exact_remainder() {
    let source = include_str!("../service.rs");
    assert!(source.contains("Vec::with_capacity(remaining.len().saturating_add(1))"));
    assert!(source.contains("unprocessed = collect_deferred_paths(pending, iterator);"));
    assert!(!source
        .contains("unprocessed.push(pending);\n                unprocessed.extend(iterator);"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fo_editor156_watcher_deferred_path_capacity_bench() {
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
        "EDITOR156_WATCHER_DEFERRED_PATH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} deferred_paths_per_build={DEFERRED_PATHS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct PendingFixture([usize; 5]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for build in 0..BUILDS_PER_SAMPLE {
        let first = PendingFixture([build; 5]);
        let remaining = std::iter::repeat_n(first, DEFERRED_PATHS_PER_BUILD - 1);
        let mut deferred = if reserve {
            Vec::with_capacity(DEFERRED_PATHS_PER_BUILD)
        } else {
            Vec::new()
        };
        deferred.push(black_box(first));
        deferred.extend(black_box(remaining));
        checksum ^= black_box(deferred.len() ^ deferred.capacity() ^ deferred[0].0[0]);
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
