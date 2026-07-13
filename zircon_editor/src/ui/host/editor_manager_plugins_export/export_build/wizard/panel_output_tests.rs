use std::sync::mpsc::channel;

use zircon_runtime_interface::export::ExportStage;

use super::*;

struct StageOutputRunner;

impl ExportWizardCommandRunner for StageOutputRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        let stage_id = command.stage.cli_id();
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: vec![
                command.stdout_banner("windows-release"),
                format!("{stage_id} stdout detail"),
            ],
            stderr_lines: if command.stage == ExportStage::Pack {
                vec!["pack stderr detail".to_string()]
            } else {
                Vec::new()
            },
        })
    }
}

#[test]
fn export_wizard_panel_template_state_projects_stage_stdout_and_stderr() {
    let plan = export_wizard_pipeline_plan(ready_options());
    let mut runner = StageOutputRunner;
    let mut emitted_events = Vec::new();
    let mut view_model = ExportWizardPanelViewModel::from_plan("export-panel-output", &plan);

    let snapshot = run_export_wizard_job(
        "export-panel-output",
        &plan,
        &mut runner,
        &ExportWizardNeverCancel,
        &mut |event| emitted_events.push(event),
    );
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);

    let (sender, receiver) = channel();
    for event in emitted_events {
        sender.send(event).expect("send wizard output event");
    }
    drop(sender);
    view_model.drain_events(&receiver);

    let rows = view_model.stage_rows();
    let pack = rows
        .iter()
        .find(|row| row.stage == ExportStage::Pack)
        .expect("Pack row should exist");
    assert!(pack
        .stdout_lines
        .iter()
        .any(|line| line == "pack stdout detail"));
    assert_eq!(pack.stderr_lines, vec!["pack stderr detail".to_string()]);

    let state = export_wizard_panel_template_state(&view_model);
    let terminal_output = state
        .slot(ExportWizardPanelSlotKind::TerminalOutput)
        .expect("terminal output slot should exist");

    assert!(terminal_output.entries.iter().any(|entry| {
        entry.key == "stage-output.pack.stdout.1"
            && entry.label == "Pack stdout"
            && entry.detail == "pack stdout detail"
            && entry.severity == ExportWizardPanelEntrySeverity::Info
    }));
    assert!(terminal_output.entries.iter().any(|entry| {
        entry.key == "stage-output.pack.stderr.0"
            && entry.label == "Pack stderr"
            && entry.detail == "pack stderr detail"
            && entry.severity == ExportWizardPanelEntrySeverity::Warning
    }));
}

fn ready_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\zircon-export\\assets\\assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\zircon_game.exe".to_string());
    options
}
