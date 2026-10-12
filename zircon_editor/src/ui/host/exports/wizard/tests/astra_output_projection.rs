use std::hint::black_box;

use super::super::{
    export_wizard_pipeline_plan, ExportWizardCommandOutputLine, ExportWizardCommandOutputStream,
    ExportWizardPipelineOptions, ExportWizardStageOutputDelta,
};
use super::*;

fn fixture(history_lines: usize) -> ExportWizardPanelViewModel {
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "projection",
        "zircon-project.toml",
        "unused-export-output",
    ));
    let mut view = ExportWizardPanelViewModel::from_plan("astra-projection", &plan);
    view.mark_job_started();
    for (index, stage) in ExportStage::ALL.into_iter().take(7).enumerate() {
        let count = (history_lines / 7 + usize::from(index < history_lines % 7)).min(512);
        if count == 0 {
            continue;
        }
        view.snapshot.stages.push(ExportWizardStageExecution {
            stage,
            command: vec!["retained-command".into()],
            exit_code: Some(0),
            stdout_lines: (0..count)
                .map(|line| format!("{stage:?} history {line}"))
                .collect(),
            stderr_lines: vec![],
            diagnostics: vec!["retained stage diagnostic".into()],
            failure: None,
            cancelled: false,
            fatal: false,
            progress: view.snapshot.progress.clone(),
        });
    }
    view.snapshot
        .diagnostics
        .push("retained job diagnostic".into());
    view
}

fn output_event(view: &ExportWizardPanelViewModel, index: usize) -> ExportWizardJobEvent {
    let mut snapshot = view.snapshot.event_header();
    snapshot.status = ExportWizardJobStatus::Running;
    snapshot.cancel_requested = false;
    ExportWizardJobEvent {
        kind: ExportWizardJobEventKind::StageOutput,
        snapshot,
        output_delta: Some(ExportWizardStageOutputDelta {
            stage: ExportStage::Report,
            output: ExportWizardCommandOutputLine {
                stream: if index % 3 == 0 {
                    ExportWizardCommandOutputStream::Stderr
                } else {
                    ExportWizardCommandOutputStream::Stdout
                },
                line: format!("current output {index}"),
            },
            progress: view.snapshot.progress.clone(),
        }),
        coalesced_output_events: (index / 4) as u64,
    }
}

// Retained pre-change implementation is the semantic and performance oracle.
fn legacy_apply(view: &mut ExportWizardPanelViewModel, event: ExportWizardJobEvent) {
    view.latest_event_kind = Some(event.kind);
    view.coalesced_output_events = view
        .coalesced_output_events
        .max(event.coalesced_output_events);
    let mut snapshot = match event.output_delta {
        Some(delta) => {
            let mut snapshot = view.snapshot.clone();
            snapshot.job_id = event.snapshot.job_id;
            snapshot.profile = event.snapshot.profile;
            snapshot.out = event.snapshot.out;
            snapshot.status = event.snapshot.status;
            snapshot.fatal = event.snapshot.fatal;
            snapshot.cancel_requested = event.snapshot.cancel_requested;
            snapshot.apply_stage_output(delta.stage, delta.output, delta.progress);
            snapshot
        }
        None => event.snapshot,
    };
    if view.snapshot.cancel_requested && !snapshot.cancel_requested && !snapshot.is_terminal() {
        snapshot.cancel_requested = true;
        if matches!(
            snapshot.status,
            ExportWizardJobStatus::Pending | ExportWizardJobStatus::Running
        ) {
            snapshot.status = ExportWizardJobStatus::Cancelling;
        }
    }
    view.active_job = !snapshot.is_terminal();
    view.snapshot = snapshot;
    view.event_count += 1;
}

#[test]
fn astra_m19_output_projection_matches_legacy_with_cancel_and_terminal_transitions() {
    for history in [0, 1, 1_000, 10_000] {
        let mut actual = fixture(history);
        let mut expected = actual.clone();
        for index in 0..1_600 {
            if index == 17 {
                actual.mark_cancel_requested();
                expected.mark_cancel_requested();
            }
            let event = output_event(&actual, index);
            legacy_apply(&mut expected, event.clone());
            actual.apply_event(event);
            assert_eq!(actual, expected);
        }
        for status in [
            ExportWizardJobStatus::Finished,
            ExportWizardJobStatus::Failed,
            ExportWizardJobStatus::Cancelled,
        ] {
            let mut snapshot = actual.snapshot.clone();
            snapshot.status = status;
            snapshot.cancel_requested = status == ExportWizardJobStatus::Cancelled;
            snapshot.fatal = status == ExportWizardJobStatus::Failed;
            let event = ExportWizardJobEvent {
                kind: terminal_event_kind(status).unwrap(),
                snapshot,
                output_delta: None,
                coalesced_output_events: 0,
            };
            legacy_apply(&mut expected, event.clone());
            actual.apply_event(event);
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn astra_m19_output_delta_keeps_unrelated_stage_and_diagnostic_allocations() {
    let mut view = fixture(1_000);
    let stages = view.snapshot.stages.as_ptr();
    let history = view.snapshot.stages[0].stdout_lines[0].as_ptr();
    let diagnostics = view.snapshot.diagnostics[0].as_ptr();
    for index in 0..128 {
        view.apply_event(output_event(&view, index));
        assert_eq!(view.snapshot.stages.as_ptr(), stages);
        assert_eq!(view.snapshot.stages[0].stdout_lines[0].as_ptr(), history);
        assert_eq!(view.snapshot.diagnostics[0].as_ptr(), diagnostics);
    }
}

fn percentiles(samples: &mut [u128]) -> [u128; 3] {
    samples.sort_unstable();
    [50, 95, 99].map(|p| samples[(samples.len() * p).div_ceil(100) - 1])
}

fn measure(
    seed: &ExportWizardPanelViewModel,
    events: &[ExportWizardJobEvent],
    optimized: bool,
) -> u128 {
    let mut view = seed.clone();
    let events = events.to_vec();
    let started = Instant::now();
    for event in events {
        if optimized {
            view.apply_event(black_box(event));
        } else {
            legacy_apply(&mut view, black_box(event));
        }
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(view);
    elapsed
}

#[test]
#[ignore = "Windows release evidence through the coordinator"]
fn astra_m19_output_projection_release_evidence() {
    assert!(
        !cfg!(debug_assertions),
        "performance evidence requires release"
    );
    const SAMPLES: usize = 101;
    const EVENTS: usize = 32;
    for history in [0, 1, 1_000, 10_000] {
        let seed = fixture(history);
        let events = (0..EVENTS)
            .map(|index| output_event(&seed, index))
            .collect::<Vec<_>>();
        for _ in 0..8 {
            black_box(measure(&seed, &events, false));
            black_box(measure(&seed, &events, true));
        }
        let mut before = Vec::with_capacity(SAMPLES);
        let mut after = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            for optimized in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let elapsed = measure(&seed, &events, optimized);
                if optimized {
                    after.push(elapsed);
                } else {
                    before.push(elapsed);
                }
            }
        }
        let before = percentiles(&mut before);
        let after = percentiles(&mut after);
        let limit = if history < 1_000 { 105 } else { 80 };
        println!(
            "ASTRA_M19_OUTPUT_PROJECTION profile=release source_history_lines={history} retained_history_lines={} events={EVENTS} samples={SAMPLES} warmup=8 before_p50_p95_p99_ns={before:?} after_p50_p95_p99_ns={after:?} p95_limit_percent={limit}",
            seed.snapshot
                .stages
                .iter()
                .map(|stage| stage.stdout_lines.len())
                .sum::<usize>()
        );
        assert!(
            after[1] * 100 <= before[1] * limit,
            "output projection p95 gate missed"
        );
    }
}
