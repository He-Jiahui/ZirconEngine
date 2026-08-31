use std::hint::black_box;
use std::time::Instant;

use super::{
    generated_files_for_profile, generated_profile_file_capacity, ExportGeneratedFile,
    ExportProfile, ProjectManifest,
};
use crate::asset::AssetUri;
use crate::core::framework::project::ExportTargetPlatform;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 26_215;
const ANDROID_PLATFORM_FILES: usize = 16;
const GENERATED_FILES_PER_BUILD: usize = 20;

#[test]
fn optimization_batch_20260826ex_runtime193_capacity_preserves_android_generated_files() {
    let manifest = ProjectManifest::new(
        "Runtime193",
        AssetUri::parse("res://scenes/main.scene").unwrap(),
        1,
    );
    let mut profile = ExportProfile::default();
    profile.target_platform = ExportTargetPlatform::Android;

    let files = generated_files_for_profile(&manifest, &profile, &[], &[], &[]);

    assert_eq!(files.len(), 19);
    assert!(files.capacity() >= 19);
    assert_eq!(files[0].path, "Cargo.toml");
    assert_eq!(files[1].path, "src/zircon_plugins.rs");
    assert_eq!(files[2].path, "assets/zircon-project.toml");
    assert_eq!(files[3].path, "src/lib.rs");
    assert_eq!(files[18].path, "platform/android/README.md");
    assert_eq!(
        generated_profile_file_capacity(ANDROID_PLATFORM_FILES, false),
        19
    );
    assert_eq!(
        generated_profile_file_capacity(ANDROID_PLATFORM_FILES, true),
        GENERATED_FILES_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826ex_runtime193_generated_files_reserve_final_output_count() {
    let source = include_str!("../generated_files.rs");
    assert!(source.contains("let platform_files = platform_host_files("));
    assert!(source.contains("Vec::with_capacity(generated_profile_file_capacity("));
    assert!(source.contains("platform_files.len()"));
    assert!(source.contains("usize::from(has_native_dynamic_plugins)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ex_runtime193_generated_export_file_capacity_bench() {
    let file = ExportGeneratedFile {
        path: String::new(),
        purpose: String::new(),
        contents: String::new(),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&file, false));
            optimized_samples.push(measure(&file, true));
        } else {
            optimized_samples.push(measure(&file, true));
            legacy_samples.push(measure(&file, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME193_GENERATED_EXPORT_FILE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} files_per_build={GENERATED_FILES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(file: &ExportGeneratedFile, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut files = if reserve {
            Vec::with_capacity(GENERATED_FILES_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..GENERATED_FILES_PER_BUILD {
            files.push(black_box(file.clone()));
        }
        checksum ^= black_box(files.len() ^ files.capacity());
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
