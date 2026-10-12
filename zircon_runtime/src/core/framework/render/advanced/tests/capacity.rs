use std::hint::black_box;
use std::time::Instant;

use super::{
    advanced_provider_degradation_capacity, AdvancedProviderAvailability, AdvancedProviderReport,
    AdvancedProviderStatus, AdvancedRenderDegradation, AdvancedRenderDegradationReason,
    AdvancedRenderFeature, RenderCapabilitySummary,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 104_858;
const DEGRADATIONS_PER_BUILD: usize = 5;

#[test]
fn optimization_batch_20260826ew_runtime192_capacity_preserves_degradation_reasons() {
    let report = AdvancedProviderReport::from_inputs(
        AdvancedRenderFeature::VirtualGeometry,
        true,
        &RenderCapabilitySummary::default(),
        &AdvancedProviderAvailability::new(),
    );

    assert_eq!(report.status, AdvancedProviderStatus::Degraded);
    assert_eq!(report.degradations.len(), DEGRADATIONS_PER_BUILD);
    assert!(report.degradations.capacity() >= DEGRADATIONS_PER_BUILD);
    assert_eq!(
        report
            .degradations
            .iter()
            .filter(|degradation| degradation.reason
                == AdvancedRenderDegradationReason::BackendCapabilityMissing)
            .count(),
        4
    );
    assert_eq!(
        report
            .degradations
            .iter()
            .filter(|degradation| degradation.reason
                == AdvancedRenderDegradationReason::ProviderMissing)
            .count(),
        1
    );
    assert_eq!(
        advanced_provider_degradation_capacity(AdvancedRenderFeature::VirtualGeometry, false),
        DEGRADATIONS_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826ew_runtime192_provider_report_reserves_degradation_upper_bound() {
    let source = include_str!("../provider_report.rs");
    assert!(source.contains("Vec::with_capacity(advanced_provider_degradation_capacity("));
    assert!(source.contains("provider_id.is_some()"));
    assert!(source.contains("usize::from(!provider_available)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ew_runtime192_advanced_provider_degradation_capacity_bench() {
    let degradation =
        AdvancedRenderDegradation::missing_provider(AdvancedRenderFeature::VirtualGeometry);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&degradation, false));
            optimized_samples.push(measure(&degradation, true));
        } else {
            optimized_samples.push(measure(&degradation, true));
            legacy_samples.push(measure(&degradation, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME192_ADVANCED_PROVIDER_DEGRADATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} degradations_per_build={DEGRADATIONS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(degradation: &AdvancedRenderDegradation, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut degradations = if reserve {
            Vec::with_capacity(DEGRADATIONS_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..DEGRADATIONS_PER_BUILD {
            degradations.push(black_box(degradation.clone()));
        }
        checksum ^= black_box(degradations.len() ^ degradations.capacity());
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
