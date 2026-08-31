use std::collections::HashMap;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, Weak};
use std::time::Instant;

use super::fence::ConfigCommitFence;
use super::path_key::absolute_path;
use super::registry::{contains_path, PathCommitEpoch};

const BENCH_PATH_COUNT: usize = 65_536;
const BENCH_SAMPLE_PAIRS: usize = 21;
static NEXT_TEST_PATH: AtomicU64 = AtomicU64::new(1);

#[test]
fn path_commit_gate_registry_reclaims_only_after_the_last_fence_drops() {
    let path = unique_path("last-owner-reclaim");
    let normalized = absolute_path(&path);
    let first = ConfigCommitFence::register(&path).unwrap();
    let second = ConfigCommitFence::register(&path).unwrap();
    assert!(registry_contains(&normalized));

    drop(first);
    assert!(
        registry_contains(&normalized),
        "a live fence must keep the shared path gate registered"
    );

    drop(second);
    assert!(
        !registry_contains(&normalized),
        "the last fence must reclaim its dead path key"
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn path_commit_gate_registry_reclaim_release_benchmark() {
    let paths = (0..BENCH_PATH_COUNT)
        .map(|index| PathBuf::from(format!("runtime55/config-{index:08}.json")))
        .collect::<Vec<_>>();
    let retained_path_bytes = paths
        .iter()
        .map(|path| path.as_os_str().len())
        .sum::<usize>();
    let mut legacy_ns = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut legacy_final_entries = 0;
    let mut optimized_final_entries = 0;
    let mut legacy_peak_entries = 0;
    let mut optimized_peak_entries = 0;

    for sample_index in 0..BENCH_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            let sample = measure_legacy(&paths);
            legacy_ns.push(sample.elapsed_ns);
            legacy_final_entries = sample.final_entries;
            legacy_peak_entries = sample.peak_entries;

            let sample = measure_reclaimed(&paths);
            optimized_ns.push(sample.elapsed_ns);
            optimized_final_entries = sample.final_entries;
            optimized_peak_entries = sample.peak_entries;
        } else {
            let sample = measure_reclaimed(&paths);
            optimized_ns.push(sample.elapsed_ns);
            optimized_final_entries = sample.final_entries;
            optimized_peak_entries = sample.peak_entries;

            let sample = measure_legacy(&paths);
            legacy_ns.push(sample.elapsed_ns);
            legacy_final_entries = sample.final_entries;
            legacy_peak_entries = sample.peak_entries;
        }
    }

    assert_eq!(legacy_final_entries, BENCH_PATH_COUNT);
    assert_eq!(legacy_peak_entries, BENCH_PATH_COUNT);
    assert_eq!(optimized_final_entries, 0);
    assert_eq!(optimized_peak_entries, 1);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    println!(
        "FOUNDATION_PATH_GATE_RECLAIM_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank timing_gate=diagnostic_only path_count={} legacy_final_entries={} optimized_final_entries={} legacy_peak_entries={} optimized_peak_entries={} legacy_retained_path_bytes={} optimized_retained_path_bytes=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        BENCH_SAMPLE_PAIRS,
        BENCH_PATH_COUNT,
        legacy_final_entries,
        optimized_final_entries,
        legacy_peak_entries,
        optimized_peak_entries,
        retained_path_bytes,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

#[derive(Clone, Copy)]
struct RegistrySample {
    elapsed_ns: u128,
    final_entries: usize,
    peak_entries: usize,
}

fn measure_legacy(paths: &[PathBuf]) -> RegistrySample {
    let started = Instant::now();
    let mut gates = HashMap::<PathBuf, Weak<Mutex<PathCommitEpoch>>>::new();
    let mut peak_entries = 0;
    for path in paths {
        gates.insert(path.clone(), Weak::new());
        peak_entries = peak_entries.max(gates.len());
    }
    black_box(&gates);
    RegistrySample {
        elapsed_ns: started.elapsed().as_nanos(),
        final_entries: gates.len(),
        peak_entries,
    }
}

fn measure_reclaimed(paths: &[PathBuf]) -> RegistrySample {
    let started = Instant::now();
    let mut gates = HashMap::<PathBuf, Weak<Mutex<PathCommitEpoch>>>::new();
    let mut peak_entries = 0;
    for path in paths {
        gates.insert(path.clone(), Weak::new());
        peak_entries = peak_entries.max(gates.len());
        gates.remove(path);
    }
    black_box(&gates);
    RegistrySample {
        elapsed_ns: started.elapsed().as_nanos(),
        final_entries: gates.len(),
        peak_entries,
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn registry_contains(path: &Path) -> bool {
    contains_path(path)
}

fn unique_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-runtime55-{name}-{}-{}",
        std::process::id(),
        NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed)
    ))
}
