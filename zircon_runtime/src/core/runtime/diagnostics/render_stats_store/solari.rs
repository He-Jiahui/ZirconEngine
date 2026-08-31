use crate::core::framework::render::{
    RenderStats, SolariDegradationReason, SolariRuntimeDegradation, SolariRuntimeStatus,
};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = &stats.last_solari_runtime_report;
    record_bool(
        store,
        "render.solari.requested",
        frame_index,
        report.requested,
        &["render", "solari", "requested"],
    );
    record_bool(
        store,
        "render.solari.enabled",
        frame_index,
        report.enabled(),
        &["render", "solari", "enabled"],
    );
    record_bool(
        store,
        "render.solari.provider_present",
        frame_index,
        report.provider_id.is_some(),
        &["render", "solari", "provider"],
    );
    record_bool(
        store,
        "render.solari.settings.experimental_enabled",
        frame_index,
        report.settings.experimental_enabled,
        &["render", "solari", "settings"],
    );
    record_status(store, frame_index, report.status);
    record_degradations(store, frame_index, stats);
}

fn record_status(store: &mut DiagnosticStore, frame_index: u64, status: SolariRuntimeStatus) {
    record_bool(
        store,
        "render.solari.status.not_requested",
        frame_index,
        status == SolariRuntimeStatus::NotRequested,
        &["render", "solari", "status"],
    );
    record_bool(
        store,
        "render.solari.status.ready",
        frame_index,
        status == SolariRuntimeStatus::Ready,
        &["render", "solari", "status", "ready"],
    );
    record_bool(
        store,
        "render.solari.status.capability_missing",
        frame_index,
        status == SolariRuntimeStatus::CapabilityMissing,
        &["render", "solari", "status", "capability"],
    );
    record_bool(
        store,
        "render.solari.status.provider_missing",
        frame_index,
        status == SolariRuntimeStatus::ProviderMissing,
        &["render", "solari", "status", "provider"],
    );
    record_bool(
        store,
        "render.solari.status.experimental_disabled",
        frame_index,
        status == SolariRuntimeStatus::ExperimentalDisabled,
        &["render", "solari", "status", "experimental"],
    );
    record_bool(
        store,
        "render.solari.status.unavailable",
        frame_index,
        status == SolariRuntimeStatus::Unavailable,
        &["render", "solari", "status", "unavailable"],
    );
}

fn record_degradations(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let degradations = &stats.last_solari_runtime_report.degradations;
    let counts = degradation_reason_counts(degradations);
    record_count(
        store,
        "render.solari.degradation_count",
        frame_index,
        degradations.len(),
        &["render", "solari", "degradation"],
    );
    record_count(
        store,
        "render.solari.backend_capability_missing_degradation_count",
        frame_index,
        counts.backend_capability_missing,
        &["render", "solari", "degradation", "capability"],
    );
    record_count(
        store,
        "render.solari.provider_missing_degradation_count",
        frame_index,
        counts.provider_missing,
        &["render", "solari", "degradation", "provider"],
    );
    record_count(
        store,
        "render.solari.experimental_disabled_degradation_count",
        frame_index,
        counts.experimental_disabled,
        &["render", "solari", "degradation", "experimental"],
    );
    record_count(
        store,
        "render.solari.provider_unavailable_degradation_count",
        frame_index,
        counts.provider_unavailable,
        &["render", "solari", "degradation", "unavailable"],
    );
}

#[derive(Default)]
struct SolariDegradationReasonCounts {
    backend_capability_missing: usize,
    provider_missing: usize,
    experimental_disabled: usize,
    provider_unavailable: usize,
}

fn degradation_reason_counts(
    degradations: &[SolariRuntimeDegradation],
) -> SolariDegradationReasonCounts {
    let mut counts = SolariDegradationReasonCounts::default();
    for degradation in degradations {
        match degradation.reason {
            SolariDegradationReason::BackendCapabilityMissing => {
                counts.backend_capability_missing += 1;
            }
            SolariDegradationReason::ProviderMissing => counts.provider_missing += 1,
            SolariDegradationReason::ExperimentalDisabled => counts.experimental_disabled += 1,
            SolariDegradationReason::ProviderUnavailable => counts.provider_unavailable += 1,
        }
    }
    counts
}

#[cfg(test)]
mod performance_tests {
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
        let source = include_str!("solari.rs");
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
                            .filter(|reason| {
                                **reason == SolariDegradationReason::ExperimentalDisabled
                            })
                            .count(),
                    )
                    .wrapping_add(
                        reasons
                            .iter()
                            .filter(|reason| {
                                **reason == SolariDegradationReason::ProviderUnavailable
                            })
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
}
