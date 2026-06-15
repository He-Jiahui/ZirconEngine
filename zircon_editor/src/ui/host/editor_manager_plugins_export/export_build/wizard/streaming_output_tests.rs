use zircon_runtime::plugin::ExportPipelineStage;

use super::*;

#[derive(Default)]
struct StreamingRunner {
    seen_stages: Vec<ExportPipelineStage>,
}

impl ExportWizardCommandRunner for StreamingRunner {
    fn run(
        &mut self,
        _command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        panic!("streaming runner should be driven through run_with_output");
    }

    fn run_with_output(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut dyn FnMut(ExportWizardCommandOutputLine),
    ) -> Result<ExportWizardCommandExecution, String> {
        self.seen_stages.push(command.stage);
        let stage_id = export_pipeline_stage_cli_id(command.stage);
        let stdout_lines = vec![
            command.stdout_banner("windows-release"),
            format!("report=D:\\zircon-export\\stages\\{stage_id}\\report.json"),
            r#""fatal": false,"#.to_string(),
        ];
        let stderr_lines = if command.stage == ExportPipelineStage::Pack {
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
fn export_wizard_job_runner_streams_stage_output_before_stage_finished() {
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
    assert_eq!(runner.seen_stages, export_pipeline_stages().to_vec());

    let validate_output_index = events
        .iter()
        .position(|event| {
            event.kind == ExportWizardJobEventKind::StageOutput
                && event.snapshot.current_stage == Some(ExportPipelineStage::Validate)
        })
        .expect("Validate should emit streamed output");
    let validate_finished_index = events
        .iter()
        .position(|event| {
            event.kind == ExportWizardJobEventKind::StageFinished
                && event.snapshot.current_stage == Some(ExportPipelineStage::Validate)
        })
        .expect("Validate should finish after output");
    assert!(
        validate_output_index < validate_finished_index,
        "StageOutput must arrive before StageFinished for retained UI polling"
    );

    let validate_output = &events[validate_output_index].snapshot;
    assert_eq!(
        validate_output
            .progress
            .snapshot(ExportPipelineStage::Validate)
            .expect("Validate progress should exist")
            .kind,
        ExportStageProgressKind::Running
    );
    assert!(validate_output
        .live_stage_outputs
        .iter()
        .any(|output| output.stage == ExportPipelineStage::Validate
            && output
                .stdout_lines
                .iter()
                .any(|line| line == "zircon_export stage=Validate profile=windows-release")));

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
        .find(|row| row.stage == ExportPipelineStage::Validate)
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
        .find(|row| row.stage == ExportPipelineStage::Pack)
        .expect("Pack row should exist after finish");
    assert_eq!(
        pack_row.stderr_lines,
        vec!["pack streaming stderr".to_string()]
    );
}

fn ready_export_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options
}
