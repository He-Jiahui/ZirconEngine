use std::hint::black_box;
use std::time::{Duration, Instant};

use super::PreparedProductBuildToolchain;
use crate::build::product_build::{ProductBuildSdkSource, ProductBuildToolchain};
use crate::build::receipt::ToolchainSet;

#[test]
#[ignore = "release-only performance evidence"]
fn shared_batch_toolchain_capture_performance_evidence() {
    const PRODUCT_COUNT: usize = 4;
    const FILE_BYTES: usize = 256 * 1024;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 50;

    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-shared-toolchain-bench-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    for (index, name) in ["cargo.exe", "rustc.exe", "link.exe", "sdk.lib"]
        .into_iter()
        .enumerate()
    {
        std::fs::write(directory.join(name), vec![index as u8; FILE_BYTES + index]).unwrap();
    }
    let source = ProductBuildToolchain {
        cargo_path: directory.join("cargo.exe"),
        rustc_path: directory.join("rustc.exe"),
        linker_path: Some(directory.join("link.exe")),
        sdk_files: vec![ProductBuildSdkSource {
            logical_name: "sdk-lib".to_string(),
            source_path: directory.join("sdk.lib"),
        }],
    };
    let environment_digests = (0..PRODUCT_COUNT)
        .map(|index| format!("{index:064X}"))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_toolchain_capture(&source, &environment_digests, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_toolchain_capture(&source, &environment_digests, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    std::fs::remove_dir_all(directory).unwrap();

    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);

    println!(
        "TOOLING15_SHARED_BATCH_TOOLCHAIN_CAPTURE_BENCH_V1 products={PRODUCT_COUNT} file_bytes={FILE_BYTES} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 50%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 50%"
    );
}

fn measure_toolchain_capture(
    source: &ProductBuildToolchain,
    environment_digests: &[String],
    baseline_first: bool,
) -> ((Vec<ToolchainSet>, Duration), (Vec<ToolchainSet>, Duration)) {
    let mut baseline_sources = vec![source.clone(); environment_digests.len()];
    let mut candidate_source = source.clone();
    let mut measure_baseline = || {
        let started = Instant::now();
        let toolchains = baseline_sources
            .iter_mut()
            .zip(environment_digests)
            .map(|(source, environment_digest)| {
                PreparedProductBuildToolchain::open(source)
                    .unwrap()
                    .receipt_toolchain(environment_digest.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        (black_box(toolchains), started.elapsed())
    };
    let mut measure_candidate = || {
        let started = Instant::now();
        let prepared = PreparedProductBuildToolchain::open(&mut candidate_source).unwrap();
        let toolchains = environment_digests
            .iter()
            .map(|environment_digest| {
                prepared
                    .receipt_toolchain(environment_digest.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        (black_box(toolchains), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}
