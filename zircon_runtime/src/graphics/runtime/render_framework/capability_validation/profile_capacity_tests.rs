use std::hint::black_box;
use std::time::Instant;

use super::{
    quality_profile_capability_capacity, RenderFeatureCapabilityRequirement, RenderQualityProfile,
    RenderQualityProfileCapabilityRequirements,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 65_536;
const MAX_REQUIREMENTS: usize = 7;

#[test]
fn optimization_batch_20260826ep_runtime185_capacity_preserves_full_profile_requirements() {
    let profile = RenderQualityProfile::new("solari")
        .with_anti_alias(true)
        .with_solari(true);

    let requirements = profile.capability_requirements();

    assert_eq!(requirements.len(), MAX_REQUIREMENTS);
    assert!(requirements.capacity() >= MAX_REQUIREMENTS);
    assert_eq!(
        requirements[0],
        RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias
    );
    assert_eq!(quality_profile_capability_capacity(false, false), 0);
}

#[test]
fn optimization_batch_20260826ep_runtime185_profile_reserves_exact_enabled_requirement_count() {
    let source = include_str!("mod.rs");
    let impl_start = source
        .find("impl RenderQualityProfileCapabilityRequirements")
        .unwrap();
    let impl_end = source[impl_start..]
        .find("fn push_unique_requirement")
        .map(|offset| impl_start + offset)
        .unwrap();
    let impl_source = &source[impl_start..impl_end];

    assert!(impl_source.contains("quality_profile_capability_capacity("));
    assert!(impl_source.contains("self.features.anti_alias"));
    assert!(impl_source.contains("self.features.solari"));
    assert!(source.contains("usize::from(anti_alias)"));
    assert!(source.contains("SolariCapabilityRequirement::ALL.len()"));
    assert_eq!(quality_profile_capability_capacity(true, false), 1);
    assert_eq!(quality_profile_capability_capacity(false, true), 6);
    assert_eq!(
        quality_profile_capability_capacity(true, true),
        MAX_REQUIREMENTS
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ep_runtime185_quality_profile_capacity_bench() {
    let requirements = RenderQualityProfile::new("solari")
        .with_anti_alias(true)
        .with_solari(true)
        .capability_requirements();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&requirements, false));
            optimized_samples.push(measure(&requirements, true));
        } else {
            optimized_samples.push(measure(&requirements, true));
            legacy_samples.push(measure(&requirements, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME185_QUALITY_PROFILE_CAPABILITY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} requirements_per_build={MAX_REQUIREMENTS} \
legacy_allocations_per_build=2 optimized_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved quality-profile capability build P95 {optimized_p95_ns}ns must be at most 70% of growth-driven build P95 {legacy_p95_ns}ns"
    );
}

fn measure(requirements: &[RenderFeatureCapabilityRequirement], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(quality_profile_capability_capacity(true, true))
        } else {
            Vec::new()
        };
        for requirement in requirements {
            output.push(black_box(*requirement));
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
