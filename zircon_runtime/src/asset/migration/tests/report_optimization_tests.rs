use std::fmt::Write as _;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::{
    AssetMigrationChange, AssetMigrationIssue, AssetMigrationIssueKind, AssetMigrationMode,
    AssetMigrationReport,
};

#[test]
fn runtime04_report_format_preallocates_known_row_capacity() {
    let source = include_str!("../report.rs");
    let implementation = source
        .split("#[cfg(test)]\nmod optimization_tests")
        .next()
        .expect("report production implementation");
    assert!(implementation.contains("String::with_capacity"));
    assert!(implementation.contains("self.changed_files.len()"));
    assert!(implementation.contains("self.issues.len()"));
}

#[test]
fn runtime04_report_format_text_preserves_legacy_output() {
    let report = sample_report(4, 3);

    assert_eq!(report.format_text(), legacy_format_text(&report));
}

#[test]
#[ignore = "managed Runtime04 performance evidence"]
fn runtime04_report_format_capacity_performance_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const ITERATIONS: usize = 32;

    let report = sample_report(256, 256);
    for _ in 0..2 {
        black_box(measure_legacy(&report));
        black_box(measure_optimized(&report));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(&report), measure_optimized(&report))
        } else {
            let optimized_ns = measure_optimized(&report);
            (measure_legacy(&report), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(2),
        "report formatting must stay within the bounded legacy comparison guard"
    );
    println!(
        "RUNTIME04_MIGRATION_REPORT_FORMAT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} changed_files={} issues={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples={} optimized_samples={} reserved_rows={} order=alternating_legacy_first_even",
        report.changed_files.len(),
        report.issues.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
        report.changed_files.len() + report.issues.len(),
    );
}

fn sample_report(changed_count: usize, issue_count: usize) -> AssetMigrationReport {
    let mut report = AssetMigrationReport::new(AssetMigrationMode::DryRun);
    report.set_scanned_files(changed_count + issue_count);
    for index in 0..changed_count {
        report.push_change(AssetMigrationChange::new(
            PathBuf::from(format!("assets/changed-{index:04}.toml")),
            index,
        ));
    }
    for index in 0..issue_count {
        report.push_issue(AssetMigrationIssue::new(
            AssetMigrationIssueKind::MissingPath,
            Some(PathBuf::from(format!("assets/issue-{index:04}.toml"))),
            format!("missing path {index}"),
        ));
    }
    report
}

fn measure_legacy(report: &AssetMigrationReport) -> u128 {
    let started = Instant::now();
    for _ in 0..32 {
        black_box(legacy_format_text(report));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(report: &AssetMigrationReport) -> u128 {
    let started = Instant::now();
    for _ in 0..32 {
        black_box(report.format_text());
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_format_text(report: &AssetMigrationReport) -> String {
    let mut output = String::new();
    write!(
        output,
        "migrate-assets mode={:?} scanned={} changed={} issues={} applied={}",
        report.mode,
        report.scanned_files,
        report.changed_files.len(),
        report.issues.len(),
        report.applied
    )
    .expect("writing to String cannot fail");
    for change in &report.changed_files {
        write!(
            output,
            "\nchange path={} references={}",
            change.path.display(),
            change.reference_count
        )
        .expect("writing to String cannot fail");
    }
    for issue in &report.issues {
        write!(output, "\nissue kind={:?} path=", issue.kind)
            .expect("writing to String cannot fail");
        if let Some(path) = &issue.path {
            write!(output, "{}", path.display()).expect("writing to String cannot fail");
        } else {
            output.push_str("<project>");
        }
        write!(output, " message={}", issue.message).expect("writing to String cannot fail");
    }
    output
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
