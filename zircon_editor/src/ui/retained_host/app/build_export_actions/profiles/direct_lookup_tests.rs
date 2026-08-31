use std::hint::black_box;
use std::time::Instant;

use super::{desktop_export_profile, desktop_export_profiles, DESKTOP_EXPORT_PROFILE_NAMES};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 10_000;
const BENCHMARK_PROFILE_NAME: &str = "headless_server";

#[test]
fn optimization_batch_20260829al_editor257_direct_lookup_matches_the_profile_catalog() {
    let catalog = desktop_export_profiles();
    assert_eq!(catalog.len(), DESKTOP_EXPORT_PROFILE_NAMES.len());
    for profile_name in DESKTOP_EXPORT_PROFILE_NAMES {
        let expected = catalog
            .iter()
            .find(|profile| profile.name == profile_name)
            .expect("catalog profile");
        assert_eq!(
            desktop_export_profile(profile_name).as_ref(),
            Some(expected)
        );
    }
    assert!(desktop_export_profile("unknown_profile").is_none());
}

#[test]
fn optimization_batch_20260829al_editor257_single_lookup_does_not_build_the_catalog() {
    let source = include_str!("../profiles.rs");
    let lookup = source
        .split("fn desktop_export_profile(")
        .nth(1)
        .expect("desktop export profile lookup")
        .split("fn desktop_client_export_profile")
        .next()
        .expect("desktop export profile lookup body");

    assert!(lookup.contains("match profile_name"));
    assert!(!lookup.contains("desktop_export_profiles()"));
    assert!(!lookup.contains(".find("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829al_editor257_direct_desktop_export_profile_lookup_bench() {
    assert_eq!(
        desktop_export_profile(BENCHMARK_PROFILE_NAME),
        legacy_desktop_export_profile(BENCHMARK_PROFILE_NAME)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR257_DIRECT_DESKTOP_EXPORT_PROFILE_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} legacy_profiles_constructed_per_build=8 \
optimized_profiles_constructed_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_desktop_export_profile(
    profile_name: &str,
) -> Option<zircon_runtime::core::framework::project::ExportProfile> {
    desktop_export_profiles()
        .into_iter()
        .find(|profile| profile.name == profile_name)
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let profile = if optimized {
            desktop_export_profile(black_box(BENCHMARK_PROFILE_NAME))
        } else {
            legacy_desktop_export_profile(black_box(BENCHMARK_PROFILE_NAME))
        }
        .expect("benchmark profile");
        checksum = checksum.wrapping_add(black_box(profile).name.len());
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
