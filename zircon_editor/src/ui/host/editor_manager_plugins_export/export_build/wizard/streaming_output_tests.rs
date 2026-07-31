use zircon_runtime_interface::export::ExportStage;

use super::*;

#[derive(Default)]
struct StreamingRunner {
    seen_stages: Vec<ExportStage>,
}

impl ExportWizardCommandRunner for StreamingRunner {
    fn run(
        &mut self,
        _command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        panic!("streaming runner should be driven through run_with_output");
    }

    fn run_with_output(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.seen_stages.push(command.stage);
        let stage_id = command.stage.cli_id();
        let stdout_lines = vec![
            command.stdout_banner("windows-release"),
            format!("report=D:\\zircon-export\\stages\\{stage_id}\\report.json"),
            r#""fatal": false,"#.to_string(),
        ];
        let stderr_lines = if command.stage == ExportStage::Pack {
            vec!["pack streaming stderr".to_string()]
        } else {
            Vec::new()
        };

        for line in &stdout_lines {
            emit_output(ExportWizardCommandOutputLine {
                stream: ExportWizardCommandOutputStream::Stdout,
                line: line.clone(),
            });
        }
        for line in &stderr_lines {
            emit_output(ExportWizardCommandOutputLine {
                stream: ExportWizardCommandOutputStream::Stderr,
                line: line.clone(),
            });
        }

        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines,
            stderr_lines,
        })
    }
}

#[test]
fn stage_output_event_carries_delta_without_accumulated_snapshot() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let mut runner = StreamingRunner::default();
    let mut events = Vec::new();

    let snapshot = run_export_wizard_job(
        "export-streaming-output",
        &plan,
        &mut runner,
        &ExportWizardNeverCancel,
        &mut |event| events.push(event),
    );

    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
    assert_eq!(runner.seen_stages, ExportStage::ALL.to_vec());

    let validate_output_index = events
        .iter()
        .position(|event| {
            event.kind == ExportWizardJobEventKind::StageOutput
                && event.snapshot.current_stage == Some(ExportStage::Validate)
        })
        .expect("Validate should emit streamed output");
    let validate_finished_index = events
        .iter()
        .position(|event| {
            event.kind == ExportWizardJobEventKind::StageFinished
                && event.snapshot.current_stage == Some(ExportStage::Validate)
        })
        .expect("Validate should finish after output");
    assert!(
        validate_output_index < validate_finished_index,
        "StageOutput must arrive before StageFinished for retained UI polling"
    );

    let validate_event = &events[validate_output_index];
    let validate_output = &validate_event.snapshot;
    assert!(validate_output.stages.is_empty());
    assert!(validate_output.live_stage_outputs.is_empty());
    assert!(validate_output.diagnostics.is_empty());
    let validate_delta = validate_event
        .output_delta
        .as_ref()
        .expect("StageOutput should carry one output delta");
    assert_eq!(validate_delta.stage, ExportStage::Validate);
    assert_eq!(
        validate_delta.output,
        ExportWizardCommandOutputLine {
            stream: ExportWizardCommandOutputStream::Stdout,
            line: "zircon_export stage=Validate profile=windows-release".to_string(),
        }
    );
    assert_eq!(
        validate_delta
            .progress
            .snapshot(ExportStage::Validate)
            .expect("Validate progress should exist")
            .kind,
        ExportStageProgressKind::Running
    );

    let mut view_model = ExportWizardPanelViewModel::from_plan("export-streaming-output", &plan);
    for event in events.iter().take(validate_output_index + 1).cloned() {
        view_model.apply_event(event);
    }
    assert_eq!(
        view_model.latest_event_kind(),
        Some(ExportWizardJobEventKind::StageOutput)
    );
    let validate_row = view_model
        .stage_rows()
        .into_iter()
        .find(|row| row.stage == ExportStage::Validate)
        .expect("Validate row should exist while running");
    assert_eq!(validate_row.progress_kind, ExportStageProgressKind::Running);
    assert!(validate_row
        .stdout_lines
        .iter()
        .any(|line| line == "zircon_export stage=Validate profile=windows-release"));

    for event in events.into_iter().skip(validate_output_index + 1) {
        view_model.apply_event(event);
    }
    assert_eq!(
        view_model.snapshot().status,
        ExportWizardJobStatus::Finished
    );
    assert!(view_model.snapshot().live_stage_outputs.is_empty());
    let pack_row = view_model
        .stage_rows()
        .into_iter()
        .find(|row| row.stage == ExportStage::Pack)
        .expect("Pack row should exist after finish");
    assert_eq!(
        pack_row.stderr_lines,
        vec!["pack streaming stderr".to_string()]
    );
}

#[test]
fn view_model_drain_is_budgeted() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let event_snapshot = ExportWizardJobState::new("event-drain-budget", &plan)
        .into_snapshot()
        .event_header();
    let (sender, receiver) = std::sync::mpsc::channel();
    for _ in 0..256 {
        sender
            .send(ExportWizardJobEvent {
                kind: ExportWizardJobEventKind::Started,
                snapshot: event_snapshot.clone(),
                output_delta: None,
                coalesced_output_events: 0,
            })
            .expect("view model test receiver should remain connected");
    }

    let mut view_model = ExportWizardPanelViewModel::from_plan("event-drain-budget", &plan);
    let drained = view_model.drain_events(&receiver);
    assert!((1..=64).contains(&drained));
    assert!(
        receiver.try_recv().is_ok(),
        "one drain must retain queued work"
    );
}

fn ready_export_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options
}
