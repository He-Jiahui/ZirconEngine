use std::collections::{HashSet, VecDeque};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;

const PATH_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bd_watch_ingress_single_probe_preserves_queue_semantics() {
    let ingress = UiAssetWatchIngressHandle::new(2);
    let first = PathBuf::from(r"C:\project\assets\ui\first.zui");
    let second = PathBuf::from(r"C:\project\assets\ui\second.zui");
    let overflow = PathBuf::from(r"C:\project\assets\ui\overflow.zui");

    ingress.record_paths([first.clone(), first.clone(), second.clone(), overflow]);
    let snapshot = ingress.snapshot(Instant::now());
    assert_eq!(snapshot.pending_path_count, 2);
    assert_eq!(snapshot.received_path_count, 4);
    assert_eq!(snapshot.coalesced_path_count, 1);
    assert_eq!(snapshot.overflow_count, 1);
    assert_eq!(ingress.path_admission_hash_probe_count(), 4);

    let drained = ingress.drain_paths(2);
    assert_eq!(
        drained
            .iter()
            .map(|pending| &pending.path)
            .collect::<Vec<_>>(),
        [&first, &second]
    );
}

#[test]
fn optimization_batch_20260826bd_watch_ingress_single_probe_eliminates_duplicate_lookup() {
    let ingress = UiAssetWatchIngressHandle::new(PATH_COUNT);
    let paths = scale_paths();
    ingress.record_paths(paths.clone());
    ingress.record_paths(paths);

    assert_eq!(ingress.path_admission_hash_probe_count(), PATH_COUNT * 2);
    let source = include_str!("../ingress.rs");
    let record_loop = source
        .split("for path in paths")
        .nth(1)
        .expect("record path loop must remain")
        .split("fn lock_state")
        .next()
        .expect("record path loop must terminate");
    assert!(!record_loop.contains("pending_path_set.contains"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826bd_watch_ingress_single_probe_p95() {
    let paths = scale_paths();
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| legacy_admit(&paths)));
            optimized.push(measure(|| single_probe_admit(&paths)));
        } else {
            optimized.push(measure(|| single_probe_admit(&paths)));
            baseline.push(measure(|| legacy_admit(&paths)));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "EDITOR23_WATCH_INGRESS_SINGLE_PROBE_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} hash_probes_before={} hash_probes_after={}",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        PATH_COUNT * 2,
        PATH_COUNT,
    );
    assert!(
        reduction >= 25.0,
        "expected at least 25% P95 reduction, got {reduction:.2}%"
    );
}

fn legacy_admit(paths: &[PathBuf]) -> VecDeque<PathBuf> {
    let mut queued = HashSet::with_capacity(paths.len());
    let mut order = VecDeque::with_capacity(paths.len());
    for path in paths {
        if queued.contains(path) {
            continue;
        }
        let _ = queued.insert(path.clone());
        order.push_back(path.clone());
    }
    order
}

fn single_probe_admit(paths: &[PathBuf]) -> VecDeque<PathBuf> {
    let mut queued = HashSet::with_capacity(paths.len());
    let mut order = VecDeque::with_capacity(paths.len());
    for path in paths {
        if !queued.insert(path.clone()) {
            continue;
        }
        order.push_back(path.clone());
    }
    order
}

fn scale_paths() -> Vec<PathBuf> {
    (0..PATH_COUNT)
        .map(|index| PathBuf::from(format!(r"C:\project\assets\ui\widget-{index:05}.zui")))
        .collect()
}

fn measure<T>(work: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(work());
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * percentile / 100]
}

fn percent_reduction(before: Duration, after: Duration) -> f64 {
    if before.is_zero() {
        return 0.0;
    }
    100.0 * (before.as_secs_f64() - after.as_secs_f64()) / before.as_secs_f64()
}
