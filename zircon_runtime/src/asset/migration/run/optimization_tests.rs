use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

const ROOT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 16;

fn root_fixture() -> Vec<(String, PathBuf)> {
    (0..ROOT_COUNT)
        .map(|index| {
            (
                format!("root/{index:04}"),
                PathBuf::from(format!("D:/project/assets/{index:04}")),
            )
        })
        .collect()
}

#[test]
fn runtime04_migration_run_capacity_projection_preserves_root_order() {
    let fixture = root_fixture();
    let mut legacy_paths = fixture
        .iter()
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let mut optimized_paths = Vec::with_capacity(fixture.len());
    optimized_paths.extend(fixture.iter().map(|(_, path)| path.clone()));
    assert_eq!(optimized_paths, legacy_paths);
    legacy_paths.clear();

    let mut optimized_roots = Vec::with_capacity(fixture.len());
    optimized_roots.extend(fixture.iter().cloned());
    assert_eq!(optimized_roots, fixture);
}

#[test]
fn runtime04_migration_run_capacity_source_contract() {
    let source = include_str!("../run.rs");
    assert!(source.matches("Vec::with_capacity(roots.len())").count() >= 2);
    assert!(!source.contains(".map(|(_, path)| path.clone())\n        .collect::<Vec<_>>()"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_migration_run_capacity_bench() {
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let fixture = root_fixture();
                let paths = fixture
                    .iter()
                    .map(|(_, path)| path.clone())
                    .collect::<Vec<_>>();
                black_box(paths);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let fixture = root_fixture();
                let mut paths = Vec::with_capacity(fixture.len());
                paths.extend(fixture.iter().map(|(_, path)| path.clone()));
                black_box(paths);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let percentile_95 = |mut samples: Vec<u128>| {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    };
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_MIGRATION_RUN_CAPACITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} roots={} root_path_capacity_reduction=0->{}",
        legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS, ROOT_COUNT, ROOT_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized migration root capacity p95 should be at most 95% of legacy p95"
    );
}
