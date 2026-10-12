use super::super::super::{
    export_wizard_pipeline_plan, run_export_wizard_job, EditorExportBuildError,
    ExportWizardCommandExecution, ExportWizardCommandOutputLine, ExportWizardCommandOutputStream,
    ExportWizardCommandRunner, ExportWizardJobSnapshot, ExportWizardJobState,
    ExportWizardJobStatus, ExportWizardNeverCancel, ExportWizardPipelineOptions,
    ExportWizardPipelinePlan, ExportWizardPipelineStageCommand, ExportWizardStageOutputDelta,
};
use super::*;

fn plan() -> ExportWizardPipelinePlan {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "admission",
        "zircon-project.toml",
        "unused-export-output",
    );
    options.source_asset_manifest = Some("unused-assets.json".into());
    options.host_executable = Some("unused-host.exe".into());
    export_wizard_pipeline_plan(options)
}

fn output_event(
    snapshot: &ExportWizardJobSnapshot,
    stage: ExportStage,
    line: usize,
) -> ExportWizardJobEvent {
    ExportWizardJobEvent {
        kind: ExportWizardJobEventKind::StageOutput,
        snapshot: snapshot.event_header(),
        output_delta: Some(ExportWizardStageOutputDelta {
            stage,
            output: ExportWizardCommandOutputLine {
                stream: if line % 2 == 0 {
                    ExportWizardCommandOutputStream::Stdout
                } else {
                    ExportWizardCommandOutputStream::Stderr
                },
                line: format!("line {line}"),
            },
            progress: snapshot.progress.clone(),
        }),
        coalesced_output_events: 0,
    }
}

#[test]
fn astra_m22_consumption_restores_admission_for_every_stage() {
    let snapshot = ExportWizardJobState::new("responsive", &plan()).into_snapshot();
    for stage in ExportStage::ALL {
        for source_events in [0, 1, 1_000, 10_000] {
            let (sender, receiver) =
                crossbeam_channel::bounded(EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY);
            let mut coalesced = 0;
            for line in 0..source_events {
                let event = output_event(&snapshot, stage, line);
                let expected_delta = event.output_delta.clone();
                send_job_event(&sender, event, &mut coalesced);
                let received = receiver
                    .try_recv()
                    .expect("consumed capacity must be reusable");
                assert_eq!(received.output_delta, expected_delta);
                assert_eq!(received.coalesced_output_events, 0);
            }
            assert_eq!(coalesced, 0);
            assert!(receiver.is_empty());
        }
    }
}

#[test]
fn astra_m22_partial_drain_releases_only_consumed_capacity() {
    let snapshot = ExportWizardJobState::new("partial-drain", &plan()).into_snapshot();
    let (sender, receiver) = crossbeam_channel::bounded(EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY);
    let mut coalesced = 0;
    for line in 0..MAX_PENDING_OUTPUT_EVENTS + 7 {
        send_job_event(
            &sender,
            output_event(&snapshot, ExportStage::Pack, line),
            &mut coalesced,
        );
    }
    assert_eq!(receiver.len(), MAX_PENDING_OUTPUT_EVENTS);
    assert_eq!(coalesced, 7);
    for _ in 0..5 {
        receiver.try_recv().unwrap();
    }
    for line in 0..6 {
        send_job_event(
            &sender,
            output_event(&snapshot, ExportStage::Pack, line),
            &mut coalesced,
        );
    }
    assert_eq!(receiver.len(), MAX_PENDING_OUTPUT_EVENTS);
    assert_eq!(coalesced, 8);
    let pending = receiver.try_iter().collect::<Vec<_>>();
    let resumed = &pending[pending.len() - 5..];
    assert!(resumed
        .iter()
        .all(|event| event.coalesced_output_events == 7));
    assert_eq!(
        resumed
            .last()
            .unwrap()
            .output_delta
            .as_ref()
            .unwrap()
            .output
            .line,
        "line 4"
    );
}

struct OutputRunner;

impl ExportWizardCommandRunner for OutputRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: std::iter::once(command.stdout_banner("admission"))
                .chain((0..1_000).map(|line| format!("output {line}")))
                .collect(),
            stderr_lines: vec![],
        })
    }
}

#[test]
fn astra_m22_real_pipeline_keeps_all_controls_without_a_consumer() {
    let (sender, receiver) = crossbeam_channel::bounded(EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY);
    let mut coalesced = 0;
    let mut emitted_output = 0;
    let mut expected_controls = Vec::new();
    let snapshot = run_export_wizard_job(
        "absent-consumer",
        &plan(),
        &mut OutputRunner,
        &ExportWizardNeverCancel,
        &mut |event| {
            if event.kind == ExportWizardJobEventKind::StageOutput {
                emitted_output += 1;
            } else {
                expected_controls.push((event.kind, event.snapshot.current_stage));
            }
            send_job_event(&sender, event, &mut coalesced);
        },
    );
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
    let events = receiver.try_iter().collect::<Vec<_>>();
    let output_count = events
        .iter()
        .filter(|event| event.kind == ExportWizardJobEventKind::StageOutput)
        .count();
    let controls = events
        .iter()
        .filter(|event| event.kind != ExportWizardJobEventKind::StageOutput)
        .map(|event| (event.kind, event.snapshot.current_stage))
        .collect::<Vec<_>>();
    assert_eq!(controls, expected_controls);
    assert_eq!(controls.len(), MAX_CONTROL_EVENTS);
    assert!(events.len() <= MAX_PENDING_OUTPUT_EVENTS + MAX_CONTROL_EVENTS);
    assert!(coalesced > 0);
    assert_eq!(coalesced, (emitted_output - output_count) as u64);
    let terminal = events.last().unwrap();
    assert_eq!(terminal.snapshot, snapshot);
    assert_eq!(terminal.coalesced_output_events, coalesced);
}

#[test]
fn astra_m22_full_output_queue_retains_each_terminal_kind() {
    let snapshot = ExportWizardJobState::new("terminal-reserve", &plan()).into_snapshot();
    for (kind, status) in [
        (
            ExportWizardJobEventKind::Finished,
            ExportWizardJobStatus::Finished,
        ),
        (
            ExportWizardJobEventKind::Failed,
            ExportWizardJobStatus::Failed,
        ),
        (
            ExportWizardJobEventKind::Cancelled,
            ExportWizardJobStatus::Cancelled,
        ),
    ] {
        let (sender, receiver) = crossbeam_channel::bounded(EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY);
        let mut coalesced = 0;
        for line in 0..10_000 {
            send_job_event(
                &sender,
                output_event(&snapshot, ExportStage::Report, line),
                &mut coalesced,
            );
        }
        let mut terminal_snapshot = snapshot.clone();
        terminal_snapshot.status = status;
        let terminal = ExportWizardJobEvent {
            kind,
            snapshot: terminal_snapshot,
            output_delta: None,
            coalesced_output_events: coalesced,
        };
        send_job_event(&sender, terminal.clone(), &mut coalesced);
        assert_eq!(coalesced, (10_000 - MAX_PENDING_OUTPUT_EVENTS) as u64);
        assert_eq!(receiver.len(), MAX_PENDING_OUTPUT_EVENTS + 1);
        assert_eq!(receiver.try_iter().last(), Some(terminal));
    }
}

#[test]
fn astra_m22_disconnected_receiver_does_not_block_completion() {
    let (sender, receiver) = crossbeam_channel::bounded(EXPORT_WIZARD_EVENT_CHANNEL_CAPACITY);
    drop(receiver);
    let mut coalesced = 0;
    let snapshot = run_export_wizard_job(
        "disconnected",
        &plan(),
        &mut OutputRunner,
        &ExportWizardNeverCancel,
        &mut |event| send_job_event(&sender, event, &mut coalesced),
    );
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
}
