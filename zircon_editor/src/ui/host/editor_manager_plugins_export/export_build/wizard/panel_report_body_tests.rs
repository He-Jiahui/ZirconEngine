use std::sync::mpsc::channel;

use zircon_runtime::plugin::ExportPipelineStage;

use super::*;

struct PipelineReportRunner;

impl ExportWizardCommandRunner for PipelineReportRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        let stage_id = export_pipeline_stage_cli_id(command.stage);
        let mut stdout_lines = vec![command.stdout_banner("windows-release")];
        if command.stage == ExportPipelineStage::Report {
            stdout_lines.push(
                "pipeline_report=D:\\zircon-export\\runtime-pipeline-report.json".to_string(),
            );
        } else {
            stdout_lines.push(format!(
                "report=D:\\zircon-export\\stages\\{stage_id}\\report.json"
            ));
        }
        stdout_lines.push(r#""fatal": false,"#.to_string());
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines,
            stderr_lines: Vec::new(),
        })
    }
}

#[test]
fn export_wizard_panel_template_state_projects_pipeline_report_body_entry() {
    let plan = export_wizard_pipeline_plan(ready_options());
    let mut view_model = ExportWizardPanelViewModel::from_plan("export-panel-report-body", &plan);

    let planned_entry = pipeline_report_entry(&view_model);
    assert_eq!(planned_entry.detail, "D:\\zircon-export\\report.json");
    assert_eq!(planned_entry.stage, Some(ExportPipelineStage::Report));
    assert_eq!(
        planned_entry.severity,
        ExportWizardPanelEntrySeverity::Neutral
    );

    let mut runner = PipelineReportRunner;
    let mut emitted_events = Vec::new();
    let snapshot = run_export_wizard_job(
        "export-panel-report-body",
        &plan,
        &mut runner,
        &ExportWizardNeverCancel,
        &mut |event| emitted_events.push(event),
    );
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);

    let (sender, receiver) = channel();
    for event in emitted_events {
        sender.send(event).expect("send wizard report event");
    }
    drop(sender);
    view_model.drain_events(&receiver);

    let runtime_entry = pipeline_report_entry(&view_model);
    assert_eq!(
        runtime_entry.detail,
        "D:\\zircon-export\\runtime-pipeline-report.json"
    );
    assert_eq!(runtime_entry.stage, Some(ExportPipelineStage::Report));
    assert_eq!(
        runtime_entry.severity,
        ExportWizardPanelEntrySeverity::Success
    );
}

fn pipeline_report_entry(view_model: &ExportWizardPanelViewModel) -> ExportWizardPanelSlotEntry {
    export_wizard_panel_template_state(view_model)
        .slot(ExportWizardPanelSlotKind::ReportBody)
        .expect("report body slot should exist")
        .entries
        .iter()
        .find(|entry| entry.key == "report.pipeline_report")
        .expect("pipeline report entry should exist")
        .clone()
}

fn ready_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\zircon-export\\assets\\assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\zircon_game.exe".to_string());
    options
}
