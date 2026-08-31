use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use zircon_runtime_interface::project::RelPath;

use super::{sort_dedup, RootRelativePhysicalIdentity};

const IDENTITY_COUNT: usize = 16_384;
const UNIQUE_IDENTITIES: usize = 512;
const PATH_COUNT: usize = 8_192;
const UNIQUE_PATHS: usize = 1_024;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 8;

fn rel(path: &str) -> RelPath {
    RelPath::parse(path).expect("fixture relative path should parse")
}

fn identity_fixture() -> Vec<RootRelativePhysicalIdentity> {
    (0..IDENTITY_COUNT)
        .rev()
        .map(|index| {
            let key = index % UNIQUE_IDENTITIES;
            RootRelativePhysicalIdentity {
                logical_root: rel(&format!("project/{key:04}")),
                root: PathBuf::from(format!("D:/assets/{key:04}")),
                relative: PathBuf::from(format!("source/{key:04}.asset")),
            }
        })
        .collect()
}

fn path_fixture() -> Vec<PathBuf> {
    (0..PATH_COUNT)
        .rev()
        .map(|index| PathBuf::from(format!("D:/assets/{:04}.asset", index % UNIQUE_PATHS)))
        .collect()
}

fn legacy_identity_sort(
    mut identities: Vec<RootRelativePhysicalIdentity>,
) -> Vec<RootRelativePhysicalIdentity> {
    identities.sort_by(|left, right| {
        left.root
            .cmp(&right.root)
            .then_with(|| left.relative.cmp(&right.relative))
            .then_with(|| left.logical_root.cmp(&right.logical_root))
    });
    identities.dedup_by(|left, right| {
        left.root == right.root
            && left.relative == right.relative
            && left.logical_root == right.logical_root
    });
    identities
}

fn legacy_path_sort(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort();
    paths.dedup();
    paths
}

fn identity_keys(identities: &[RootRelativePhysicalIdentity]) -> Vec<(PathBuf, PathBuf, RelPath)> {
    identities
        .iter()
        .map(|identity| {
            (
                identity.root.clone(),
                identity.relative.clone(),
                identity.logical_root.clone(),
            )
        })
        .collect()
}

fn percentile_95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[test]
fn runtime04_migration_scan_unstable_sort_preserves_dedup_order() {
    let expected_identities = legacy_identity_sort(identity_fixture());
    let mut optimized_identities = identity_fixture();
    optimized_identities.sort_unstable_by(|left, right| {
        left.root
            .cmp(&right.root)
            .then_with(|| left.relative.cmp(&right.relative))
            .then_with(|| left.logical_root.cmp(&right.logical_root))
    });
    optimized_identities.dedup_by(|left, right| {
        left.root == right.root
            && left.relative == right.relative
            && left.logical_root == right.logical_root
    });
    assert_eq!(
        identity_keys(&optimized_identities),
        identity_keys(&expected_identities)
    );

    let expected_paths = legacy_path_sort(path_fixture());
    let mut optimized_paths = path_fixture();
    sort_dedup(&mut optimized_paths);
    assert_eq!(optimized_paths, expected_paths);
    assert_eq!(optimized_identities.len(), UNIQUE_IDENTITIES);
    assert_eq!(optimized_paths.len(), UNIQUE_PATHS);
}

#[test]
fn runtime04_migration_scan_unstable_sort_source_contract() {
    let source = include_str!("../scan.rs");
    assert!(source.contains("children.sort_unstable_by_key"));
    assert!(source.contains("identities.sort_unstable_by"));
    assert!(source.contains("paths.sort_unstable()"));
    assert!(!source.contains("children.sort_by_key"));
    assert!(!source.contains("identities.sort_by(|left, right|"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_migration_scan_unstable_sort_bench() {
    let legacy_identity_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_identity_sort(identity_fixture()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_identity_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = identity_fixture();
                values.sort_unstable_by(|left, right| {
                    left.root
                        .cmp(&right.root)
                        .then_with(|| left.relative.cmp(&right.relative))
                        .then_with(|| left.logical_root.cmp(&right.logical_root))
                });
                values.dedup_by(|left, right| {
                    left.root == right.root
                        && left.relative == right.relative
                        && left.logical_root == right.logical_root
                });
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_path_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_path_sort(path_fixture()));
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_path_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut values = path_fixture();
                sort_dedup(&mut values);
                black_box(values);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let legacy_identity_p95 = percentile_95(legacy_identity_samples);
    let optimized_identity_p95 = percentile_95(optimized_identity_samples);
    let legacy_path_p95 = percentile_95(legacy_path_samples);
    let optimized_path_p95 = percentile_95(optimized_path_samples);
    println!(
        "RUNTIME04_MIGRATION_SCAN_UNSTABLE_SORT_BENCH_V1 identity_legacy_p95_ns={} identity_optimized_p95_ns={} path_legacy_p95_ns={} path_optimized_p95_ns={} samples={} iterations={} identities={} unique_identities={} paths={} unique_paths={} stable_sorts=3->0",
        legacy_identity_p95,
        optimized_identity_p95,
        legacy_path_p95,
        optimized_path_p95,
        SAMPLE_COUNT,
        ITERATIONS,
        IDENTITY_COUNT,
        UNIQUE_IDENTITIES,
        PATH_COUNT,
        UNIQUE_PATHS,
    );
    assert!(
        optimized_identity_p95.saturating_mul(100) <= legacy_identity_p95.saturating_mul(95),
        "optimized identity sort p95 should be at most 95% of legacy p95"
    );
    assert!(
        optimized_path_p95.saturating_mul(100) <= legacy_path_p95.saturating_mul(95),
        "optimized path sort p95 should be at most 95% of legacy p95"
    );
}
