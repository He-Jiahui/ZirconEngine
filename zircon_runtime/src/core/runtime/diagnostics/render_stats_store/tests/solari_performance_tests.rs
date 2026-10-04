use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::{
    RenderCapabilityKind, RenderCapabilityMismatchDetail, SolariRuntimeDegradation,
};

use super::*;

#[test]
fn optimization_batch_eg_solari_degradation_counts_preserve_every_reason() {
    let degradations = vec![
        SolariRuntimeDegradation::missing_capability(RenderCapabilityMismatchDetail::new(
            RenderCapabilityKind::InlineRayQuery,
        )),
        SolariRuntimeDegradation::missing_provider(),
        SolariRuntimeDegradation::experimental_disabled(),
        SolariRuntimeDegradation::provider_unavailable("provider failed"),
        SolariRuntimeDegradation::missing_provider(),
    ];

    let counts = degradation_reason_counts(&degradations);

    assert_eq!(counts.backend_capability_missing, 1);
    assert_eq!(counts.provider_missing, 2);
    assert_eq!(counts.experimental_disabled, 1);
    assert_eq!(counts.provider_unavailable, 1);
}

#[test]
fn optimization_batch_eg_solari_degradation_recording_uses_one_scan() {
    let source = include_str!("../solari.rs");
    let implementation = source
        .split("fn record_degradations")
        .nth(1)
        .expect("Solari degradation recorder")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded production implementation");

    assert!(implementation.contains("degradation_reason_counts(degradations)"));
    assert!(!implementation.contains(".filter("));
    assert!(!implementation.contains(".count()"));
}

#[test]
#[ignore = "release-only Solari degradation single-scan benchmark"]
fn optimization_batch_eg_solari_degradation_single_scan_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const ROWS: usize = 8_192;
    const SCANS_PER_SAMPLE: usize = 256;

    fn measure_legacy(reasons: &[SolariDegradationReason]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..SCANS_PER_SAMPLE {
            let reasons = black_box(reasons);
            checksum = checksum
                .wrapping_add(
                    reasons
                        .iter()
                        .filter(|reason| {
                            **reason == SolariDegradationReason::BackendCapabilityMissing
                        })
                        .count(),
                )
                .wrapping_add(
                    reasons
                        .iter()
                        .filter(|reason| **reason == SolariDegradationReason::ProviderMissing)
                        .count(),
                )
                .wrapping_add(
                    reasons
                        .iter()
                        .filter(|reason| **reason == SolariDegradationReason::ExperimentalDisabled)
                        .count(),
                )
                .wrapping_add(
                    reasons
                        .iter()
                        .filter(|reason| **reason == SolariDegradationReason::ProviderUnavailable)
                        .count(),
                );
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(reasons: &[SolariDegradationReason]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..SCANS_PER_SAMPLE {
            let mut counts = [0usize; 4];
            for reason in black_box(reasons) {
                match reason {
                    SolariDegradationReason::BackendCapabilityMissing => counts[0] += 1,
                    SolariDegradationReason::ProviderMissing => counts[1] += 1,
                    SolariDegradationReason::ExperimentalDisabled => counts[2] += 1,
                    SolariDegradationReason::ProviderUnavailable => counts[3] += 1,
                }
            }
            checksum = checksum.wrapping_add(counts.into_iter().sum::<usize>());
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

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let reasons = (0..ROWS)
        .map(|index| match index % 4 {
            0 => SolariDegradationReason::BackendCapabilityMissing,
            1 => SolariDegradationReason::ProviderMissing,
            2 => SolariDegradationReason::ExperimentalDisabled,
            _ => SolariDegradationReason::ProviderUnavailable,
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&reasons));
            optimized_samples.push(measure_optimized(&reasons));
        } else {
            optimized_samples.push(measure_optimized(&reasons));
            legacy_samples.push(measure_legacy(&reasons));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME441_SOLARI_DEGRADATION_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             rows={ROWS} scans_per_sample={SCANS_PER_SAMPLE} pair_order=alternating_legacy_even \
             legacy_passes_per_scan=4 optimized_passes_per_scan=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(60),
        "single-pass Solari degradation counting must reduce P95 by at least 40%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
