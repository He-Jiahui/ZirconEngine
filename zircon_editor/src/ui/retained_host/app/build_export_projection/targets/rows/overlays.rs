use std::path::Path;

use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;

use super::super::super::super::{build_export_actions, RetainedEditorHost};
use super::super::diagnostics::prepend_desktop_export_output_diagnostic;

pub(in super::super) fn apply_target_overlays(
    host: &RetainedEditorHost,
    project_root: &Path,
    job_snapshots: &[build_export_actions::DesktopExportJobSnapshot],
    target: &mut BuildExportTargetViewData,
) {
    let profile_name = target.profile_name.as_str();
    let output_root = host.effective_desktop_export_output_root(project_root, profile_name);
    let summary = host.desktop_export_reports.get(profile_name);
    let job = job_snapshots
        .iter()
        .find(|job| job.profile_name == profile_name);
    let diagnostics = prepend_desktop_export_output_diagnostic(
        output_root.as_path(),
        target.diagnostics.to_string(),
    );

    target.diagnostics = diagnostics.into();
    if let Some(summary) = summary {
        build_export_actions::apply_summary_to_target(target, summary);
    }
    if let Some(job) = job {
        build_export_actions::apply_job_snapshot_to_target(target, job);
    }
}

#[cfg(test)]
mod performance_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_eg_export_profile_lookups_borrow_shared_string() {
        let source = include_str!("overlays.rs");
        let implementation = source
            .split("fn apply_target_overlays")
            .nth(1)
            .expect("target overlay implementation")
            .split("#[cfg(test)]")
            .next()
            .expect("bounded production implementation");

        assert!(implementation.contains("let profile_name = target.profile_name.as_str();"));
        assert!(!implementation.contains("target.profile_name.to_string()"));
    }

    #[test]
    #[ignore = "release-only borrowed export profile lookup benchmark"]
    fn optimization_batch_eg_borrowed_export_profile_lookup_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 16_384;

        fn measure_legacy(reports: &BTreeMap<String, usize>, fixture: &str) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..LOOKUPS_PER_SAMPLE {
                let profile_name = black_box(fixture).to_string();
                checksum = checksum.wrapping_add(
                    *black_box(reports.get(profile_name.as_str())).expect("fixture profile"),
                );
                black_box(profile_name);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(reports: &BTreeMap<String, usize>, fixture: &str) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..LOOKUPS_PER_SAMPLE {
                let profile_name = black_box(fixture);
                checksum = checksum
                    .wrapping_add(*black_box(reports.get(profile_name)).expect("fixture profile"));
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

        let fixture = format!("desktop_windows_{}shipping", "profile_".repeat(64));
        let reports = BTreeMap::from([(fixture.clone(), 7usize)]);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&reports, &fixture));
                optimized_samples.push(measure_optimized(&reports, &fixture));
            } else {
                optimized_samples.push(measure_optimized(&reports, &fixture));
                legacy_samples.push(measure_legacy(&reports, &fixture));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR369_BORROWED_EXPORT_PROFILE_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             lookups_per_sample={LOOKUPS_PER_SAMPLE} profile_bytes={} pair_order=alternating_legacy_even \
             legacy_profile_allocations_per_sample={LOOKUPS_PER_SAMPLE} optimized_profile_allocations_per_sample=0 \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            fixture.len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "borrowed export profile lookup must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
