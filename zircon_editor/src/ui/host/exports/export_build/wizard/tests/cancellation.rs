use crate::core::jobs::CancellationToken;
use zircon_runtime_interface::export::ExportStage;

use super::*;

struct SharedCancelSignal {
    cancel: CancellationToken,
}

impl ExportWizardCancelSignal for SharedCancelSignal {
    fn is_cancel_requested(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

struct InStageCancellingRunner {
    cancel: CancellationToken,
    seen_stages: Vec<ExportStage>,
    saw_cancel_before_stage_output: bool,
    saw_cancel_after_stage_output: bool,
}

impl InStageCancellingRunner {
    fn new(cancel: CancellationToken) -> Self {
        Self {
            cancel,
            seen_stages: Vec::new(),
            saw_cancel_before_stage_output: false,
            saw_cancel_after_stage_output: false,
        }
    }
}

impl ExportWizardCommandRunner for InStageCancellingRunner {
    fn run(
        &mut self,
        _command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        panic!("in-stage cancellation runner should use run_with_output_and_cancel");
    }

    fn run_with_output_and_cancel(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
        should_cancel: &mut (dyn FnMut() -> bool + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.seen_stages.push(command.stage);
        self.saw_cancel_before_stage_output = should_cancel();

        let stdout_lines = vec![command.stdout_banner("windows-release")];
        for line in &stdout_lines {
            emit_output(ExportWizardCommandOutputLine {
                stream: ExportWizardCommandOutputStream::Stdout,
                line: line.clone(),
            });
        }

        self.cancel.cancel();
        self.saw_cancel_after_stage_output = should_cancel();

        Ok(ExportWizardCommandExecution {
            exit_code: Some(9),
            stdout_lines,
            stderr_lines: vec!["process terminated after cancel request".to_string()],
        })
    }
}

#[test]
fn export_wizard_job_runner_cancels_during_active_stage_without_failing() {
    let cancel = CancellationToken::default();
    let cancel_signal = SharedCancelSignal {
        cancel: cancel.clone(),
    };
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let mut runner = InStageCancellingRunner::new(cancel);
    let mut events = Vec::new();

    let snapshot = run_export_wizard_job(
        "export-in-stage-cancel",
        &plan,
        &mut runner,
        &cancel_signal,
        &mut |event| events.push(event),
    );

    assert_eq!(snapshot.status, ExportWizardJobStatus::Cancelled);
    assert!(snapshot.cancel_requested);
    assert!(!snapshot.fatal);
    assert_eq!(runner.seen_stages, vec![ExportStage::Validate]);
    assert!(!runner.saw_cancel_before_stage_output);
    assert!(runner.saw_cancel_after_stage_output);
    assert!(!events
        .iter()
        .any(|event| event.kind == ExportWizardJobEventKind::Failed));
    assert_eq!(
        events.last().map(|event| event.kind),
        Some(ExportWizardJobEventKind::Cancelled)
    );

    let stage_execution = snapshot
        .stages
        .first()
        .expect("cancelled active stage should still be recorded");
    assert_eq!(stage_execution.stage, ExportStage::Validate);
    assert!(stage_execution.cancelled);
    assert!(!stage_execution.fatal);
    assert_eq!(stage_execution.exit_code, Some(9));
    assert!(stage_execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("cancelled during process execution")));
    assert!(!stage_execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("exited with code 9")));

    let mut view_model = ExportWizardPanelViewModel::from_plan("export-in-stage-cancel", &plan);
    for event in events {
        view_model.apply_event(event);
    }
    let validate_row = view_model
        .stage_rows()
        .into_iter()
        .find(|row| row.stage == ExportStage::Validate)
        .expect("Validate row should be present after cancellation");
    assert_eq!(validate_row.progress_kind, ExportStageProgressKind::Running);
    assert!(validate_row
        .stdout_lines
        .iter()
        .any(|line| line == "zircon_export stage=Validate profile=windows-release"));
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
