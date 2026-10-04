use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const CHECKS_PER_SAMPLE: usize = 160_000;
const EXTENSIONS: [&str; 4] = [" PNG ", " .KtX2 ", " DDS", ".AstC "];
const SUFFIXES: [&str; 4] = [
    " .SKELETON.ZRANIM ",
    ".Clip.ZRANIM ",
    " .Scene.TOML",
    ".PREFAB.TOML ",
];

#[test]
fn importer_descriptor_normalization_contract_in_place_extensions() {
    assert_eq!(normalize_extension_owned("  .PnG  ".to_string()), "png");
    let descriptor =
        AssetImporterDescriptor::new("test.normalized", "test.plugin", AssetKind::Texture, 1)
            .with_source_extensions([" .KtX2 ".to_string()]);
    assert_eq!(descriptor.source_extensions, ["ktx2"]);
}

#[test]
fn importer_descriptor_normalization_contract_in_place_full_suffixes() {
    assert_eq!(
        normalize_full_suffix_owned("  Skeleton.ZRANIM  ".to_string()),
        ".skeleton.zranim"
    );
    let descriptor =
        AssetImporterDescriptor::new("test.normalized", "test.plugin", AssetKind::Texture, 1)
            .with_full_suffixes([" Scene.TOML ".to_string()]);
    assert_eq!(descriptor.full_suffixes, [".scene.toml"]);
}

#[test]
#[ignore = "release performance gate"]
fn importer_descriptor_normalization_performance_release_extensions() {
    let (legacy_samples, optimized_samples) =
        alternating_samples(measure_legacy_extensions, measure_in_place_extensions);
    report_normalization_performance(
        "plugins07_importer_extension_in_place_normalization",
        CHECKS_PER_SAMPLE * 2,
        CHECKS_PER_SAMPLE,
        &legacy_samples,
        &optimized_samples,
    );
}

#[test]
#[ignore = "release performance gate"]
fn importer_descriptor_normalization_performance_release_full_suffixes() {
    let (legacy_samples, optimized_samples) =
        alternating_samples(measure_legacy_suffixes, measure_in_place_suffixes);
    report_normalization_performance(
        "plugins07_importer_suffix_in_place_normalization",
        CHECKS_PER_SAMPLE * 2,
        CHECKS_PER_SAMPLE,
        &legacy_samples,
        &optimized_samples,
    );
}

fn measure_legacy_extensions() -> u128 {
    measure_owned_normalization(&EXTENSIONS, |value| {
        let owned = value.to_string();
        normalize_extension(black_box(&owned))
    })
}

fn measure_in_place_extensions() -> u128 {
    measure_owned_normalization(&EXTENSIONS, |value| {
        normalize_extension_owned(value.to_string())
    })
}

fn measure_legacy_suffixes() -> u128 {
    measure_owned_normalization(&SUFFIXES, |value| {
        let owned = value.to_string();
        normalize_full_suffix(black_box(&owned))
    })
}

fn measure_in_place_suffixes() -> u128 {
    measure_owned_normalization(&SUFFIXES, |value| {
        normalize_full_suffix_owned(value.to_string())
    })
}

fn measure_owned_normalization(values: &[&str], mut normalize: impl FnMut(&str) -> String) -> u128 {
    let started = Instant::now();
    let mut total_len = 0_usize;
    for check in 0..CHECKS_PER_SAMPLE {
        let normalized = normalize(black_box(values[check % values.len()]));
        total_len += black_box(normalized.len());
        black_box(normalized);
    }
    black_box(total_len);
    started.elapsed().as_nanos().max(1)
}

fn alternating_samples(
    mut legacy: impl FnMut() -> u128,
    mut optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(legacy());
        black_box(optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn report_normalization_performance(
    name: &str,
    legacy_allocations_per_sample: usize,
    optimized_allocations_per_sample: usize,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {name} sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={legacy_allocations_per_sample} optimized_allocations_per_sample={optimized_allocations_per_sample} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        improvement_percent >= 25,
        "in-place importer descriptor normalization must improve P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
