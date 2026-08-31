use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::stable_label_for_path;

const CHECKS_PER_SAMPLE: usize = 1024;
const SAMPLE_PAIRS: usize = 31;
const PATH_COMPONENTS: usize = 256;

fn legacy_stable_label_for_path(asset_root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(asset_root).unwrap_or(path);
    let normalized = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    format!("asset-scan://{normalized}")
}

fn measure(asset_root: &Path, path: &Path, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut bytes = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let label = if optimized {
            stable_label_for_path(black_box(asset_root), black_box(path))
        } else {
            legacy_stable_label_for_path(black_box(asset_root), black_box(path))
        };
        bytes += label.len();
        black_box(label);
    }
    black_box(bytes);
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

#[test]
fn optimization_batch_20260829bs_runtime346_stable_labels_preserve_results() {
    for (root, path) in [
        ("assets", "assets/shaders/main.wgsl"),
        ("assets", "outside/main.wgsl"),
        ("assets", "assets"),
        ("assets", "assets/\u{4f8b}/main.wgsl"),
    ] {
        assert_eq!(
            stable_label_for_path(Path::new(root), Path::new(path)),
            legacy_stable_label_for_path(Path::new(root), Path::new(path)),
            "{root:?} -> {path:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bs_runtime346_stable_label_streams_components() {
    let source = include_str!("../paths.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("fn stable_label_for_path")
        .expect("label function")
        .1
        .split_once("fn wgsl_files_for_document")
        .expect("WGSL boundary")
        .0;
    assert!(function.contains("String::with_capacity"));
    assert!(function.contains("for component in relative.components()"));
    assert!(!function.contains("collect::<Vec<_>>()"));
    assert!(!function.contains("format!"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bs_runtime346_streamed_stable_label_bench() {
    let asset_root = PathBuf::from("assets");
    let mut path = asset_root.clone();
    for _ in 0..PATH_COMPONENTS {
        path.push("segment");
    }
    path.push("main.wgsl");
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&asset_root, &path, false));
            candidate.push(measure(&asset_root, &path, true));
        } else {
            candidate.push(measure(&asset_root, &path, true));
            baseline.push(measure(&asset_root, &path, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME346_STREAMED_STABLE_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} path_components={PATH_COMPONENTS} baseline_owned_allocations_per_check=3 candidate_owned_allocations_per_check=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
