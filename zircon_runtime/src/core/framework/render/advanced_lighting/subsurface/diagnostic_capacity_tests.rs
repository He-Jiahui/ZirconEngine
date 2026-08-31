use std::hint::black_box;
use std::time::Instant;

use super::{
    resolve_subsurface_profile_table, subsurface_diagnostic_capacity, SubsurfaceProfileData,
    SubsurfaceProfileDiagnostic, Vec3, ZR_SSS_MAX_PROFILES,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const DIAGNOSTICS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826eo_runtime184_capacity_preserves_overflow_diagnostics() {
    let profiles = (0..DIAGNOSTICS_PER_BUILD)
        .map(|profile_id| {
            SubsurfaceProfileData::new(profile_id as u32, Vec3::ZERO, Vec3::ZERO, 1.0)
        })
        .collect::<Vec<_>>();

    let table = resolve_subsurface_profile_table(&profiles);

    assert_eq!(table.profiles.len(), ZR_SSS_MAX_PROFILES);
    assert_eq!(
        table.diagnostics.len(),
        DIAGNOSTICS_PER_BUILD - ZR_SSS_MAX_PROFILES
    );
    assert!(table.diagnostics.capacity() >= profiles.len());
    assert_eq!(table.diagnostics[0].profile_id, ZR_SSS_MAX_PROFILES as u32);
}

#[test]
fn optimization_batch_20260826eo_runtime184_sss_reserves_only_overflow_batches() {
    let source = include_str!("../subsurface.rs");
    let resolver_start = source.find("fn resolve_subsurface_profile_table").unwrap();
    let resolver_end = source[resolver_start..]
        .find("pub fn burley_radial_pdf")
        .map(|offset| resolver_start + offset)
        .unwrap();
    let resolver_source = &source[resolver_start..resolver_end];

    assert!(resolver_source
        .contains("Vec::with_capacity(subsurface_diagnostic_capacity(profiles.len()))"));
    assert!(source.contains("profile_count > ZR_SSS_MAX_PROFILES"));
    assert_eq!(subsurface_diagnostic_capacity(ZR_SSS_MAX_PROFILES), 0);
    assert_eq!(
        subsurface_diagnostic_capacity(DIAGNOSTICS_PER_BUILD),
        DIAGNOSTICS_PER_BUILD
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eo_runtime184_sss_diagnostic_capacity_bench() {
    let diagnostic = SubsurfaceProfileDiagnostic {
        profile_id: 17,
        message: String::new(),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&diagnostic, false));
            optimized_samples.push(measure(&diagnostic, true));
        } else {
            optimized_samples.push(measure(&diagnostic, true));
            legacy_samples.push(measure(&diagnostic, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME184_SSS_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} diagnostics_per_build={DIAGNOSTICS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved SSS diagnostic build P95 {optimized_p95_ns}ns must be at most 70% of growth-driven build P95 {legacy_p95_ns}ns"
    );
}

fn measure(diagnostic: &SubsurfaceProfileDiagnostic, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(subsurface_diagnostic_capacity(DIAGNOSTICS_PER_BUILD))
        } else {
            Vec::new()
        };
        for _ in 0..DIAGNOSTICS_PER_BUILD {
            output.push(black_box(diagnostic.clone()));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
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
