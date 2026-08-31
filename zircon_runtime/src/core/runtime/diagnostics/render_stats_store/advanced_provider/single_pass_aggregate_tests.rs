use std::hint::black_box;
use std::time::Instant;

use super::{advanced_provider_aggregate_counts, AdvancedProviderAggregateCounts};
use crate::core::framework::render::{
    AdvancedProviderReport, AdvancedProviderStatus, AdvancedRenderDegradation,
    AdvancedRenderDegradationReason, AdvancedRenderFeature,
};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 5_000;
const REPORT_COUNT: usize = 512;

#[test]
fn optimization_batch_20260829ao_runtime315_single_pass_matches_multi_scan_counts() {
    let reports = reports();

    assert_eq!(
        advanced_provider_aggregate_counts(&reports),
        legacy_counts(&reports)
    );
}

#[test]
fn optimization_batch_20260829ao_runtime315_record_reports_uses_one_aggregate() {
    let source = include_str!("../advanced_provider.rs");
    let record_reports = source
        .split("fn record_reports")
        .nth(1)
        .expect("advanced provider report recorder")
        .split("fn record_feature")
        .next()
        .expect("advanced provider report recorder body");
    let aggregate = source
        .split("fn advanced_provider_aggregate_counts")
        .nth(1)
        .expect("advanced provider aggregate")
        .split("struct AdvancedProviderFeaturePaths")
        .next()
        .expect("advanced provider aggregate body");

    assert_eq!(
        record_reports
            .matches("advanced_provider_aggregate_counts(")
            .count(),
        1
    );
    assert!(aggregate.contains("for report in reports"));
    assert!(!record_reports.contains("reports.iter().filter"));
    assert!(!source.contains("fn degradation_reason_count"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ao_runtime315_single_pass_provider_diagnostic_aggregation_bench() {
    let reports = reports();
    assert_eq!(
        advanced_provider_aggregate_counts(&reports),
        legacy_counts(&reports)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&reports, false));
            optimized_samples.push(measure(&reports, true));
        } else {
            optimized_samples.push(measure(&reports, true));
            legacy_samples.push(measure(&reports, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME315_SINGLE_PASS_PROVIDER_DIAGNOSTIC_AGGREGATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} reports_per_build={REPORT_COUNT} \
legacy_top_level_passes_per_build=7 optimized_top_level_passes_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn reports() -> Vec<AdvancedProviderReport> {
    (0..REPORT_COUNT)
        .map(|index| {
            let feature = if index % 2 == 0 {
                AdvancedRenderFeature::VirtualGeometry
            } else {
                AdvancedRenderFeature::HybridGlobalIllumination
            };
            let status = match index % 3 {
                0 => AdvancedProviderStatus::NotRequested,
                1 => AdvancedProviderStatus::Ready,
                _ => AdvancedProviderStatus::Degraded,
            };
            AdvancedProviderReport {
                feature,
                requested: status != AdvancedProviderStatus::NotRequested,
                provider_id: None,
                status,
                degradations: vec![
                    AdvancedRenderDegradation {
                        feature,
                        reason: AdvancedRenderDegradationReason::BackendCapabilityMissing,
                        missing_capability: None,
                    },
                    AdvancedRenderDegradation::missing_provider(feature),
                ],
            }
        })
        .collect()
}

fn legacy_counts(reports: &[AdvancedProviderReport]) -> AdvancedProviderAggregateCounts {
    AdvancedProviderAggregateCounts {
        requested: reports.iter().filter(|report| report.requested).count(),
        ready: reports
            .iter()
            .filter(|report| report.status == AdvancedProviderStatus::Ready)
            .count(),
        degraded: reports
            .iter()
            .filter(|report| report.status == AdvancedProviderStatus::Degraded)
            .count(),
        enabled: reports.iter().filter(|report| report.enabled()).count(),
        degradations: reports.iter().map(|report| report.degradations.len()).sum(),
        missing_capability_degradations: reports
            .iter()
            .flat_map(|report| &report.degradations)
            .filter(|degradation| {
                degradation.reason == AdvancedRenderDegradationReason::BackendCapabilityMissing
            })
            .count(),
        missing_provider_degradations: reports
            .iter()
            .flat_map(|report| &report.degradations)
            .filter(|degradation| {
                degradation.reason == AdvancedRenderDegradationReason::ProviderMissing
            })
            .count(),
    }
}

fn measure(reports: &[AdvancedProviderReport], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let counts = black_box(if optimized {
            advanced_provider_aggregate_counts(black_box(reports))
        } else {
            legacy_counts(black_box(reports))
        });
        checksum = checksum
            .wrapping_add(counts.requested)
            .wrapping_add(counts.ready)
            .wrapping_add(counts.degraded)
            .wrapping_add(counts.enabled)
            .wrapping_add(counts.degradations)
            .wrapping_add(counts.missing_capability_degradations)
            .wrapping_add(counts.missing_provider_degradations);
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
