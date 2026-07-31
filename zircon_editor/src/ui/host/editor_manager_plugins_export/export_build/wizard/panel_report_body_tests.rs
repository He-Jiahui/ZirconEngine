use std::sync::mpsc::channel;

use zircon_runtime_interface::export::ExportStage;

use super::*;

struct PipelineReportRunner;

#[test]
fn panel_report_json_is_parsed_once_for_all_summaries() {
    let projection = include_str!("panel_projection.rs");
    assert_eq!(projection.matches("serde_json::from_str").count(), 1);
}

impl ExportWizardCommandRunner for PipelineReportRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        let stage_id = command.stage.cli_id();
        let mut stdout_lines = vec![command.stdout_banner("windows-release")];
        if command.stage == ExportStage::Report {
            stdout_lines.push(
                "pipeline_report=D:\\zircon-export\\runtime-pipeline-report.json".to_string(),
            );
            stdout_lines.extend([
                "{".to_string(),
                r#"  "stage": "Report","#.to_string(),
                r#"  "profile": "windows-release","#.to_string(),
                r#"  "fatal": false,"#.to_string(),
                r#"  "diagnostics": ["curly { text } is diagnostic payload"],"#.to_string(),
                r#"  "out": "D:\\zircon-export","#.to_string(),
                r#"  "export_plan": {"#.to_string(),
                r#"    "strategies": ["#.to_string(),
                r#"      "library_embed","#.to_string(),
                r#"      "source_template""#.to_string(),
                r#"    ],"#.to_string(),
                r#"    "required_stages": ["#.to_string(),
                r#"      "validate","#.to_string(),
                r#"      "source_template","#.to_string(),
                r#"      "compile_host","#.to_string(),
                r#"      "cook_assets","#.to_string(),
                r#"      "pack","#.to_string(),
                r#"      "platform_bundle""#.to_string(),
                r#"    ],"#.to_string(),
                r#"    "completed_stages": ["#.to_string(),
                r#"      "validate","#.to_string(),
                r#"      "source_template","#.to_string(),
                r#"      "compile_host","#.to_string(),
                r#"      "cook_assets","#.to_string(),
                r#"      "pack","#.to_string(),
                r#"      "platform_bundle","#.to_string(),
                r#"      "report""#.to_string(),
                r#"    ],"#.to_string(),
                r#"    "unsupported_strategies": []"#.to_string(),
                r#"  },"#.to_string(),
                r#"  "native_plugins_payload": {"#.to_string(),
                r#"    "bundle_path": "D:\\zircon-export\\bundle\\windows-release\\plugins","#.to_string(),
                r#"    "content_hash": "native-payload-sha256","#.to_string(),
                r#"    "file_count": 4,"#.to_string(),
                r#"    "package_count": 2,"#.to_string(),
                r#"    "materialized_packages": ["#.to_string(),
                r#"      {"#.to_string(),
                r#"        "package_id": "animation","#.to_string(),
                r#"        "destination": "D:\\zircon-export\\bundle\\windows-release\\plugins\\animation","#.to_string(),
                r#"        "loadable_artifact_count": 1,"#.to_string(),
                r#"        "loadable_artifacts": ["plugins/animation/native/zircon_plugin_animation.dll"]"#.to_string(),
                r#"      },"#.to_string(),
                r#"      {"#.to_string(),
                r#"        "package_id": "physics","#.to_string(),
                r#"        "destination": "D:\\zircon-export\\bundle\\windows-release\\plugins\\physics","#.to_string(),
                r#"        "loadable_artifact_count": 1,"#.to_string(),
                r#"        "loadable_artifacts": ["plugins/physics/native/zircon_plugin_physics.dll"]"#.to_string(),
                r#"      }"#.to_string(),
                r#"    ]"#.to_string(),
                r#"  }"#.to_string(),
                "}".to_string(),
            ]);
        } else {
            stdout_lines.push(format!(
                "report=D:\\zircon-export\\stages\\{stage_id}\\report.json"
            ));
            stdout_lines.push(r#""fatal": false,"#.to_string());
        }
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
    assert_eq!(planned_entry.stage, Some(ExportStage::Report));
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
    assert_eq!(runtime_entry.stage, Some(ExportStage::Report));
    assert_eq!(
        runtime_entry.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let strategies = report_body_entry(&view_model, "report.export_plan.strategies");
    assert_eq!(strategies.label, "Export Strategies");
    assert_eq!(strategies.detail, "library_embed, source_template");
    assert_eq!(strategies.stage, Some(ExportStage::Report));
    assert_eq!(strategies.severity, ExportWizardPanelEntrySeverity::Info);

    let required_stages = report_body_entry(&view_model, "report.export_plan.required_stages");
    assert_eq!(required_stages.label, "Required Stages");
    assert_eq!(
        required_stages.detail,
        "validate, source_template, compile_host, cook_assets, pack, platform_bundle"
    );

    let completed_stages = report_body_entry(&view_model, "report.export_plan.completed_stages");
    assert_eq!(completed_stages.label, "Completed Stages");
    assert_eq!(
        completed_stages.detail,
        "validate, source_template, compile_host, cook_assets, pack, platform_bundle, report"
    );
    assert_eq!(
        completed_stages.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let unsupported_strategies =
        report_body_entry(&view_model, "report.export_plan.unsupported_strategies");
    assert_eq!(unsupported_strategies.label, "Unsupported Strategies");
    assert_eq!(unsupported_strategies.detail, "none");
    assert_eq!(
        unsupported_strategies.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let native_bundle = report_body_entry(&view_model, "report.native_plugins_payload.bundle_path");
    assert_eq!(native_bundle.label, "Native Plugins Bundle");
    assert_eq!(
        native_bundle.detail,
        "D:\\zircon-export\\bundle\\windows-release\\plugins"
    );
    assert_eq!(native_bundle.stage, Some(ExportStage::Report));
    assert_eq!(
        native_bundle.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let native_package_count =
        report_body_entry(&view_model, "report.native_plugins_payload.package_count");
    assert_eq!(native_package_count.label, "Native Plugin Packages");
    assert_eq!(native_package_count.detail, "2");
    assert_eq!(
        native_package_count.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let native_file_count =
        report_body_entry(&view_model, "report.native_plugins_payload.file_count");
    assert_eq!(native_file_count.label, "Native Plugin Files");
    assert_eq!(native_file_count.detail, "4");
    assert_eq!(
        native_file_count.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let native_content_hash =
        report_body_entry(&view_model, "report.native_plugins_payload.content_hash");
    assert_eq!(native_content_hash.label, "Native Plugin Hash");
    assert_eq!(native_content_hash.detail, "native-payload-sha256");
    assert_eq!(
        native_content_hash.severity,
        ExportWizardPanelEntrySeverity::Success
    );

    let native_packages =
        report_body_entry(&view_model, "report.native_plugins_payload.package_ids");
    assert_eq!(native_packages.label, "Native Plugin Package Ids");
    assert_eq!(native_packages.detail, "animation, physics");
    assert_eq!(
        native_packages.severity,
        ExportWizardPanelEntrySeverity::Success
    );
}

fn pipeline_report_entry(view_model: &ExportWizardPanelViewModel) -> ExportWizardPanelSlotEntry {
    report_body_entry(view_model, "report.pipeline_report")
}

fn report_body_entry(
    view_model: &ExportWizardPanelViewModel,
    key: &str,
) -> ExportWizardPanelSlotEntry {
    export_wizard_panel_template_state(view_model)
        .slot(ExportWizardPanelSlotKind::ReportBody)
        .expect("report body slot should exist")
        .entries
        .iter()
        .find(|entry| entry.key == key)
        .unwrap_or_else(|| panic!("{key} entry should exist"))
        .clone()
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
