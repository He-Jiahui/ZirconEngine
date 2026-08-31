use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const MANIFEST_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bf_atlas_manifest_collection_filters_and_sorts_paths() {
    let candidates = collect_atlas_manifest_candidates([
        PathBuf::from("cache/zeta.toml"),
        PathBuf::from("cache/ignored.png"),
        PathBuf::from("cache/Alpha.TOML"),
        PathBuf::from("cache/middle.toml"),
    ]);

    assert_eq!(
        candidates,
        [
            PathBuf::from("cache/Alpha.TOML"),
            PathBuf::from("cache/middle.toml"),
            PathBuf::from("cache/zeta.toml"),
        ]
    );
}

#[test]
fn optimization_batch_20260826bf_atlas_manifest_collection_eliminates_pairwise_dedup() {
    assert_eq!(MANIFEST_COUNT * (MANIFEST_COUNT - 1) / 2, 8_386_560);

    let source = include_str!("../discovery.rs");
    let collection = source
        .split("fn collect_atlas_manifest_candidates")
        .nth(1)
        .expect("atlas manifest collection helper must exist")
        .split("#[cfg(test)]")
        .next()
        .expect("atlas manifest collection helper must terminate");
    assert!(!collection.contains("candidates.iter().any"));
    assert!(collection.contains("sort_unstable"));
}

#[test]
#[ignore = "release-only managed performance gate"]
fn optimization_batch_20260826bf_atlas_manifest_collection_p95() {
    let paths = manifest_paths(MANIFEST_COUNT);
    let mut baseline = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            baseline.push(measure(|| legacy_collect(paths.iter().cloned())));
            optimized.push(measure(|| {
                collect_atlas_manifest_candidates(paths.iter().cloned())
            }));
        } else {
            optimized.push(measure(|| {
                collect_atlas_manifest_candidates(paths.iter().cloned())
            }));
            baseline.push(measure(|| legacy_collect(paths.iter().cloned())));
        }
    }

    let baseline_p50 = percentile(&mut baseline.clone(), 50);
    let baseline_p95 = percentile(&mut baseline, 95);
    let optimized_p50 = percentile(&mut optimized.clone(), 50);
    let optimized_p95 = percentile(&mut optimized, 95);
    let reduction = percent_reduction(baseline_p95, optimized_p95);
    println!(
        "EDITOR34_ATLAS_MANIFEST_LINEAR_COLLECTION_BENCH_V1 baseline_p50_ns={} baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_reduction_percent={reduction:.2} pairwise_path_comparisons_before=8386560 pairwise_path_comparisons_after=0 manifest_classifications_after={MANIFEST_COUNT}",
        baseline_p50.as_nanos(),
        baseline_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        reduction >= 75.0,
        "expected at least 75% P95 reduction, got {reduction:.2}%"
    );
}

fn manifest_paths(count: usize) -> Vec<PathBuf> {
    (0..count)
        .rev()
        .map(|index| PathBuf::from(format!("cache/atlas-{index:05}.toml")))
        .collect()
}

fn legacy_collect(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for path in paths {
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
            && !candidates.iter().any(|candidate| candidate == &path)
        {
            candidates.push(path);
        }
    }
    candidates.sort();
    candidates
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
