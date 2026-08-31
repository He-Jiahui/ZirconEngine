use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::{normalize_project_relative_path, AutosaveSourcePath};

const SAMPLE_PAIRS: usize = 21;
const PATHS_PER_SAMPLE: usize = 16_384;
const COMPONENTS_PER_PATH: usize = 32;

#[test]
fn optimization_batch_20260826dl_editor101_autosave_source_path_preserves_validation() {
    let parsed = AutosaveSourcePath::parse("assets/ui/panels/main.zui")
        .expect("project-relative source path should parse");
    assert_eq!(parsed.as_path(), Path::new("assets/ui/panels/main.zui"));

    assert!(AutosaveSourcePath::parse("").is_err());
    assert!(AutosaveSourcePath::parse("../outside.zui").is_err());
    let dotted = AutosaveSourcePath::parse("assets/./panel.zui")
        .expect("normal component iteration should fold an internal current-directory marker");
    assert_eq!(dotted.as_path(), Path::new("assets/panel.zui"));
}

#[test]
fn optimization_batch_20260826dl_editor101_autosave_source_path_appends_without_vec() {
    let path = fixture_path();
    let normalized = normalize_project_relative_path(&path);
    assert_eq!(normalized.len(), normalized.capacity());

    let source = include_str!("../source_path.rs");
    assert!(source.contains("let normalized = normalize_project_relative_path(&path);"));
    assert!(source.contains("String::with_capacity(path.as_os_str().len())"));
    assert!(source.contains("normalized.push_str("));
    assert!(!source.contains("collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dl_editor101_autosave_source_path_direct_join_bench() {
    let path = fixture_path();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&path, legacy_normalize));
            optimized_samples.push(measure(&path, normalize_project_relative_path));
        } else {
            optimized_samples.push(measure(&path, normalize_project_relative_path));
            legacy_samples.push(measure(&path, legacy_normalize));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR101_AUTOSAVE_SOURCE_PATH_DIRECT_JOIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
paths_per_sample={PATHS_PER_SAMPLE} components_per_path={COMPONENTS_PER_PATH} \
legacy_temporary_vecs_per_sample={PATHS_PER_SAMPLE} optimized_temporary_vecs_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct autosave source path join P95 {optimized_p95_ns}ns must be at most 70% of component-vector joining P95 {legacy_p95_ns}ns"
    );
}

fn fixture_path() -> PathBuf {
    let mut path = PathBuf::new();
    for index in 0..COMPONENTS_PER_PATH {
        path.push(format!("source_{index:02}"));
    }
    path
}

fn legacy_normalize(path: &Path) -> String {
    path.components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .expect("fixture components are UTF-8")
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn measure(path: &Path, normalize: fn(&Path) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..PATHS_PER_SAMPLE {
        checksum ^= black_box(normalize(black_box(path))).len();
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
