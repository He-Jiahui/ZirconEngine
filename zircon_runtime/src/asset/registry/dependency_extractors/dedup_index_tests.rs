use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::asset::AssetUri;

const MODEL_COUNT: usize = 4_096;
const BENCH_COUNT: usize = 2_048;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826be_handwritten_dependency_index_preserves_stable_order() {
    let first = uri("res://textures/first.png");
    let second = uri("res://textures/second.png");
    let third = uri("res://textures/third.png");
    let mut dependencies = vec![first.clone(), second.clone()];

    append_unique_dependencies(
        &mut dependencies,
        vec![second.clone(), third.clone(), third.clone()],
    );

    assert_eq!(dependencies, [first, second, third]);
}

#[test]
fn optimization_batch_20260826be_handwritten_dependency_index_eliminates_pairwise_work() {
    let legacy_comparisons = MODEL_COUNT * MODEL_COUNT + MODEL_COUNT * (MODEL_COUNT - 1) / 2;
    assert_eq!(legacy_comparisons, 25_163_776);

    let source = include_str!("mod.rs");
    let append = source
        .split("fn append_unique_dependencies")
        .nth(1)
        .expect("indexed dependency append helper must exist")
        .split("pub(crate) fn handwritten_dependencies")
        .next()
        .expect("indexed dependency append helper must terminate");
    assert!(append.contains("HashSet"));
    assert!(!append.contains("dependencies.contains"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826be_handwritten_dependency_index_p95() {
    let existing = dependency_range("existing", BENCH_COUNT);
    let candidates = dependency_range("candidate", BENCH_COUNT);
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| {
                legacy_append(existing.clone(), candidates.clone())
            }));
            optimized.push(measure(|| {
                indexed_append(existing.clone(), candidates.clone())
            }));
        } else {
            optimized.push(measure(|| {
                indexed_append(existing.clone(), candidates.clone())
            }));
            baseline.push(measure(|| {
                legacy_append(existing.clone(), candidates.clone())
            }));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "RUNTIME88_HANDWRITTEN_DEPENDENCY_DEDUP_INDEX_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} pairwise_comparisons_before={} membership_probes_after={}",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        BENCH_COUNT * BENCH_COUNT + BENCH_COUNT * (BENCH_COUNT - 1) / 2,
        BENCH_COUNT,
    );
    assert!(
        reduction >= 75.0,
        "expected at least 75% P95 reduction, got {reduction:.2}%"
    );
}

fn legacy_append(mut dependencies: Vec<AssetUri>, candidates: Vec<AssetUri>) -> Vec<AssetUri> {
    for dependency in candidates {
        if !dependencies.contains(&dependency) {
            dependencies.push(dependency);
        }
    }
    dependencies
}

fn indexed_append(mut dependencies: Vec<AssetUri>, candidates: Vec<AssetUri>) -> Vec<AssetUri> {
    append_unique_dependencies(&mut dependencies, candidates);
    dependencies
}

fn dependency_range(prefix: &str, count: usize) -> Vec<AssetUri> {
    (0..count)
        .map(|index| uri(&format!("res://{prefix}/{index:05}.asset")))
        .collect()
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
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
