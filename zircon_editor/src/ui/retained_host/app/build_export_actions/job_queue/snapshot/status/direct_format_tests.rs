use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::super::{
    DesktopExportJobPhase, DesktopExportJobSnapshot, DesktopExportProgressSnapshot,
};
use super::job_pane_diagnostics;

const MARKER: &str = "EDITOR185_EXPORT_JOB_STATUS_DIRECT_FORMAT_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 8_192;

#[test]
fn optimization_batch_20260826gs_editor185_job_status_preserves_progress_and_idle_text() {
    let mut snapshot = DesktopExportJobSnapshot {
        id: 7,
        profile_name: "shipping".to_string(),
        output_root: PathBuf::from("C:/Zircon/Builds/shipping"),
        phase: DesktopExportJobPhase::Queued,
        progress: None,
    };
    assert_eq!(
        job_pane_diagnostics(&snapshot),
        "Output: C:/Zircon/Builds/shipping\nProgress: waiting for the current desktop export job"
    );

    snapshot.phase = DesktopExportJobPhase::Running;
    snapshot.progress = Some(DesktopExportProgressSnapshot {
        stage: "compile_host".to_string(),
        percent: 73,
        message: "linking runtime".to_string(),
    });
    assert_eq!(
        job_pane_diagnostics(&snapshot),
        "Output: C:/Zircon/Builds/shipping\nProgress: export backend is running\nStage: 73% compile_host - linking runtime"
    );
}

#[test]
fn optimization_batch_20260826gs_editor185_job_status_formats_final_string_directly() {
    let source = include_str!("../status.rs");
    assert!(source.contains("match snapshot.progress.as_ref()"));
    assert!(!source.contains("let mut lines = vec!["));
    assert!(!source.contains("lines.join(\"\\n\")"));
    assert!(!source.contains("lines.push(progress_pane_diagnostic(progress))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gs_editor185_export_job_status_direct_format_bench() {
    let snapshot = DesktopExportJobSnapshot {
        id: 11,
        profile_name: "shipping".repeat(8),
        output_root: PathBuf::from(format!(
            "C:/Zircon/Builds/{}/shipping",
            "nested/".repeat(24)
        )),
        phase: DesktopExportJobPhase::Running,
        progress: Some(DesktopExportProgressSnapshot {
            stage: "compile_host".repeat(8),
            percent: 87,
            message: "linking runtime artifacts ".repeat(24),
        }),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&snapshot, legacy_job_pane_diagnostics));
            optimized_samples.push(measure(&snapshot, job_pane_diagnostics));
        } else {
            optimized_samples.push(measure(&snapshot, job_pane_diagnostics));
            legacy_samples.push(measure(&snapshot, legacy_job_pane_diagnostics));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct formatting must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_job_pane_diagnostics(snapshot: &DesktopExportJobSnapshot) -> String {
    let phase = match snapshot.phase {
        DesktopExportJobPhase::Queued => "waiting for the current desktop export job",
        DesktopExportJobPhase::Running => "export backend is running",
        DesktopExportJobPhase::CancelRequested => {
            "cancel requested; backend result will be ignored when it returns"
        }
    };
    let mut lines = vec![
        format!("Output: {}", snapshot.output_root.display()),
        format!("Progress: {phase}"),
    ];
    if let Some(progress) = &snapshot.progress {
        lines.push(format!(
            "Stage: {}% {} - {}",
            progress.percent, progress.stage, progress.message
        ));
    }
    lines.join("\n")
}

fn measure(
    snapshot: &DesktopExportJobSnapshot,
    implementation: fn(&DesktopExportJobSnapshot) -> String,
) -> u64 {
    let started = Instant::now();
    let mut bytes = 0;
    for _ in 0..REPEATS {
        bytes += implementation(black_box(snapshot)).len();
    }
    black_box(bytes);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
