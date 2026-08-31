use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::{
    summary_pane_diagnostic_capacity, summary_pane_diagnostics, DesktopExportExecutionState,
    DesktopExportExecutionSummary,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const BASE_LINES_PER_BUILD: usize = 2;
const FATAL_DIAGNOSTICS_PER_BUILD: usize = 248;
const VISIBLE_DIAGNOSTICS_PER_BUILD: usize = 6;
const LINES_PER_BUILD: usize =
    BASE_LINES_PER_BUILD + FATAL_DIAGNOSTICS_PER_BUILD + VISIBLE_DIAGNOSTICS_PER_BUILD;

#[test]
fn optimization_batch_20260826ff_editor147_capacity_preserves_export_summary_diagnostics() {
    let summary = DesktopExportExecutionSummary {
        profile_name: "desktop_windows".to_string(),
        output_root: PathBuf::from("Builds/windows"),
        state: DesktopExportExecutionState::Failed,
        invoked_cargo: true,
        generated_files: 0,
        copied_packages: 0,
        fatal_diagnostics: diagnostics("fatal", 128),
        diagnostics: diagnostics("diagnostic", 128),
    };

    let rendered = summary_pane_diagnostics(&summary);
    let lines = rendered.lines().collect::<Vec<_>>();

    assert_eq!(summary_pane_diagnostic_capacity(&summary), 136);
    assert_eq!(lines.len(), 136);
    assert_eq!(
        lines[0],
        format!("Last export output: {}", summary.output_root.display())
    );
    assert_eq!(lines[1], "Last export invoked Cargo");
    assert_eq!(lines[2], "fatal-000");
    assert_eq!(lines[129], "fatal-127");
    assert_eq!(lines[130], "diagnostic-000");
    assert_eq!(lines[135], "diagnostic-005");
}

#[test]
fn optimization_batch_20260826ff_editor147_export_summary_reserves_visible_diagnostics() {
    let source = include_str!("../status.rs");
    assert!(source.contains("const SUMMARY_PANE_BASE_LINE_COUNT: usize = 2;"));
    assert!(source.contains("const SUMMARY_PANE_DIAGNOSTIC_LIMIT: usize = 6;"));
    assert!(source.contains("fn summary_pane_diagnostic_capacity("));
    assert!(source.contains("summary.fatal_diagnostics.len()"));
    assert!(source.contains(".min(SUMMARY_PANE_DIAGNOSTIC_LIMIT)"));
    assert!(source.contains(".take(SUMMARY_PANE_DIAGNOSTIC_LIMIT)"));
    assert!(source.contains("Vec::with_capacity(summary_pane_diagnostic_capacity(summary))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ff_editor147_export_summary_diagnostic_capacity_bench() {
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
        "EDITOR147_EXPORT_SUMMARY_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} base_lines={BASE_LINES_PER_BUILD} \
fatal_diagnostics={FATAL_DIAGNOSTICS_PER_BUILD} visible_diagnostics={VISIBLE_DIAGNOSTICS_PER_BUILD} \
lines_per_build={LINES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn diagnostics(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("{prefix}-{index:03}"))
        .collect()
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut lines = if reserve {
            Vec::with_capacity(LINES_PER_BUILD)
        } else {
            Vec::new()
        };
        lines.push(black_box(usize::MAX));
        lines.push(black_box(usize::MAX - 1));
        for line in 0..FATAL_DIAGNOSTICS_PER_BUILD {
            lines.push(black_box(line));
        }
        for line in 0..VISIBLE_DIAGNOSTICS_PER_BUILD {
            lines.push(black_box(line));
        }
        checksum ^= black_box(lines.len() ^ lines.capacity());
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
