use std::collections::BTreeMap;

use super::super::DesktopExportExecutionSummary;

mod cancellation;
mod enqueue;
mod polling;

fn insert_desktop_export_report(
    reports: &mut BTreeMap<String, DesktopExportExecutionSummary>,
    summary: DesktopExportExecutionSummary,
) {
    if let Some(current) = reports.get_mut(summary.profile_name.as_str()) {
        *current = summary;
    } else {
        reports.insert(summary.profile_name.clone(), summary);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::super::super::{DesktopExportExecutionState, DesktopExportExecutionSummary};
    use super::*;

    const PROFILE_NAME: &str =
        "desktop_windows_shipping_with_editor_symbols_and_representative_project_feature_set";
    const SAMPLE_PAIRS: usize = 17;
    const UPDATES_PER_SAMPLE: usize = 8_192;

    #[test]
    fn optimization_batch_fp_editor402_reuses_existing_export_report_key() {
        let source = include_str!("jobs.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("jobs production source");
        assert!(production.contains("reports.get_mut(summary.profile_name.as_str())"));

        let mut reports = BTreeMap::new();
        insert_desktop_export_report(&mut reports, summary(1));
        insert_desktop_export_report(&mut reports, summary(7));

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[PROFILE_NAME].generated_files, 7);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fp_editor402_borrowed_export_report_key_benchmark() {
        for _ in 0..4 {
            black_box(measure_existing_key(false));
            black_box(measure_existing_key(true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_existing_key(false));
                optimized_samples.push(measure_existing_key(true));
            } else {
                optimized_samples.push(measure_existing_key(true));
                legacy_samples.push(measure_existing_key(false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR402_BORROWED_EXPORT_REPORT_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} updates_per_sample={UPDATES_PER_SAMPLE} profile_name_bytes={} legacy_key_copies_per_sample={UPDATES_PER_SAMPLE} optimized_key_copies_per_sample=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
            PROFILE_NAME.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 70 / 100);
    }

    fn measure_existing_key(optimized: bool) -> u128 {
        let summaries = (0..UPDATES_PER_SAMPLE).map(summary).collect::<Vec<_>>();
        let mut reports = BTreeMap::from([(PROFILE_NAME.to_owned(), summary(0))]);
        let started = Instant::now();
        for summary in summaries {
            if optimized {
                insert_desktop_export_report(black_box(&mut reports), summary);
            } else {
                legacy_insert_desktop_export_report(black_box(&mut reports), summary);
            }
        }
        black_box(reports);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_insert_desktop_export_report(
        reports: &mut BTreeMap<String, DesktopExportExecutionSummary>,
        summary: DesktopExportExecutionSummary,
    ) {
        reports.insert(summary.profile_name.clone(), summary);
    }

    fn summary(generated_files: usize) -> DesktopExportExecutionSummary {
        DesktopExportExecutionSummary {
            profile_name: PROFILE_NAME.to_owned(),
            output_root: PathBuf::new(),
            state: DesktopExportExecutionState::Exported,
            invoked_cargo: true,
            generated_files,
            copied_packages: 0,
            diagnostics: Vec::new(),
            fatal_diagnostics: Vec::new(),
        }
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
