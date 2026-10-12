use std::hint::black_box;
use std::time::Instant;

use super::super::{export_wizard_pipeline_plan, ExportWizardPipelineOptions};
use super::*;

fn fixture(source_lines: usize) -> ExportWizardJobState {
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "terminal",
        "zircon-project.toml",
        "unused-export-output",
    ));
    let mut job = ExportWizardJobState::new("astra-terminal", &plan);
    job.begin();
    for (index, stage) in ExportStage::ALL.into_iter().enumerate() {
        let count = (source_lines / 8 + usize::from(index < source_lines % 8)).min(512);
        if count == 0 {
            continue;
        }
        let mut progress = job.snapshot.progress.clone();
        progress.push_stdout_line(&format!("zircon_export stage={stage:?} profile=terminal"));
        job.record_stage_execution(ExportWizardStageExecution {
            stage,
            command: vec!["retained-command".into()],
            exit_code: Some(0),
            stdout_lines: (0..count)
                .map(|line| format!("{stage:?} line {line}"))
                .collect(),
            stderr_lines: vec![],
            diagnostics: vec!["retained stage diagnostic".into()],
            failure: None,
            cancelled: false,
            fatal: false,
            progress,
        });
    }
    job.snapshot
        .diagnostics
        .push("retained pipeline diagnostic".into());
    job
}

fn legacy_finish(job: &mut ExportWizardJobState, progress: ExportWizardProgressState) {
    let execution = ExportWizardPipelineExecution {
        stages: job.snapshot.stages.clone(),
        progress,
        diagnostics: job.snapshot.diagnostics.clone(),
        fatal: job.snapshot.fatal,
    };
    job.finish_from_pipeline(execution);
}

#[test]
fn astra_m21_recorded_completion_preserves_result_storage_and_terminal_precedence() {
    for lines in [0, 1, 1_000, 10_000] {
        for (fatal, cancelled, status) in [
            (false, false, ExportWizardJobStatus::Finished),
            (true, false, ExportWizardJobStatus::Failed),
            (false, true, ExportWizardJobStatus::Cancelled),
            (true, true, ExportWizardJobStatus::Cancelled),
        ] {
            let mut actual = fixture(lines);
            actual.snapshot.fatal = fatal;
            actual.snapshot.cancel_requested = cancelled;
            actual.record_stage_output(
                ExportStage::Report,
                ExportWizardCommandOutputLine {
                    stream: ExportWizardCommandOutputStream::Stdout,
                    line: "uncommitted live output".into(),
                },
                actual.snapshot.progress.clone(),
            );
            let mut expected = actual.clone();
            let stages = actual.snapshot.stages.as_ptr();
            let diagnostics = actual.snapshot.diagnostics.as_ptr();
            let progress = actual.snapshot.progress.snapshots().as_ptr();
            let expected_progress = expected.snapshot.progress.clone();
            legacy_finish(&mut expected, expected_progress);
            actual.finish_recorded_stages();
            assert_eq!(actual, expected);
            assert_eq!(actual.snapshot.status, status);
            assert!(actual.snapshot.live_stage_outputs.is_empty());
            assert_eq!(actual.snapshot.stages.as_ptr(), stages);
            assert_eq!(actual.snapshot.diagnostics.as_ptr(), diagnostics);
            assert_eq!(actual.snapshot.progress.snapshots().as_ptr(), progress);
            assert_eq!(
                actual.snapshot.current_stage,
                actual.snapshot.stages.last().map(|stage| stage.stage)
            );
        }
    }
}

fn measure(seed: &ExportWizardJobState, optimized: bool) -> u128 {
    let mut jobs = (0..16)
        .map(|_| (seed.clone(), Some(seed.snapshot.progress.clone())))
        .collect::<Vec<_>>();
    let started = Instant::now();
    for (job, progress) in &mut jobs {
        let progress = progress.take().unwrap();
        if optimized {
            black_box(&mut *job).finish_recorded_stages();
            drop(progress);
        } else {
            legacy_finish(black_box(&mut *job), progress);
        }
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(jobs);
    elapsed
}

fn percentiles(samples: &mut [u128]) -> [u128; 3] {
    samples.sort_unstable();
    [50, 95, 99].map(|p| samples[(samples.len() * p).div_ceil(100) - 1])
}

#[test]
#[ignore = "Windows release evidence through the coordinator"]
fn astra_m21_terminal_snapshot_release_evidence() {
    assert!(
        !cfg!(debug_assertions),
        "performance evidence requires release"
    );
    const SAMPLES: usize = 101;
    for lines in [0, 1, 1_000, 10_000] {
        let seed = fixture(lines);
        for _ in 0..8 {
            black_box(measure(&seed, false));
            black_box(measure(&seed, true));
        }
        let mut before = Vec::with_capacity(SAMPLES);
        let mut after = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            for optimized in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let elapsed = measure(&seed, optimized);
                if optimized {
                    after.push(elapsed);
                } else {
                    before.push(elapsed);
                }
            }
        }
        let before = percentiles(&mut before);
        let after = percentiles(&mut after);
        let limit = if lines < 1_000 { 105 } else { 80 };
        println!(
            "ASTRA_M21_TERMINAL_SNAPSHOT profile=release source_lines={lines} retained_lines={} completions=16 samples={SAMPLES} warmup=8 before_p50_p95_p99_ns={before:?} after_p50_p95_p99_ns={after:?} p95_limit_percent={limit}",
            seed.snapshot
                .stages
                .iter()
                .map(|stage| stage.stdout_lines.len())
                .sum::<usize>()
        );
        assert!(
            after[1] * 100 <= before[1] * limit,
            "terminal snapshot p95 gate missed"
        );
    }
}
