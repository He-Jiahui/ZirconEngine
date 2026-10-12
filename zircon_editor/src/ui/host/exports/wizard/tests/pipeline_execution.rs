use zircon_runtime_interface::export::ExportStage;

use super::super::*;
use super::support::*;

#[test]
fn export_wizard_stage_execution_feeds_stdout_into_progress() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let command = plan.command(ExportStage::Pack).expect("pack command");
    let mut runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(0),
        stdout_lines: vec![
            command.stdout_banner("windows-release"),
            "pack=D:\\zircon-export\\stages\\pack\\assets.zrpack".to_string(),
            "report=D:\\zircon-export\\stages\\pack\\report.json".to_string(),
            r#""fatal": false,"#.to_string(),
        ],
        stderr_lines: Vec::new(),
    });
    let mut progress = ExportWizardProgressState::new();

    let execution = execute_export_wizard_stage(command, &mut runner, &mut progress);

    assert_eq!(execution.exit_code, Some(0));
    assert!(!execution.fatal);
    assert!(execution.diagnostics.is_empty());
    assert_eq!(
        execution
            .progress
            .snapshot(ExportStage::Pack)
            .expect("pack progress")
            .report_path
            .as_deref(),
        Some("D:\\zircon-export\\stages\\pack\\report.json")
    );
    assert_eq!(runner.seen_stages, vec![ExportStage::Pack]);
}

#[test]
fn export_wizard_stage_execution_preserves_report_json_diagnostics() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let command = plan.command(ExportStage::Report).expect("report command");
    let mut runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(0),
        stdout_lines: vec![
            command.stdout_banner("windows-release"),
            r#""diagnostics": ["#.to_string(),
            r#"  "validate failed","#.to_string(),
            r#"],"#.to_string(),
            r#""fatal": true,"#.to_string(),
        ],
        stderr_lines: Vec::new(),
    });
    let mut progress = ExportWizardProgressState::new();

    let execution = execute_export_wizard_stage(command, &mut runner, &mut progress);

    assert!(execution.fatal);
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic == "validate failed"));
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("reported fatal status")));
}

#[test]
fn export_wizard_pipeline_execution_stops_on_missing_inputs_before_process_run() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();

    let execution = execute_export_wizard_pipeline(&plan, &mut runner);

    assert_eq!(
        execution.stages.last().expect("stopped stage").stage,
        ExportStage::CookAssets
    );
    assert!(execution.fatal);
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("source_asset_manifest")));
    assert_eq!(
        runner.seen_stages,
        vec![
            ExportStage::Validate,
            ExportStage::SourceTemplate,
            ExportStage::NativeDynamic,
            ExportStage::CompileHost,
        ]
    );
}

#[test]
fn export_wizard_pipeline_execution_stops_on_plan_diagnostics_before_process_run() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();

    let execution = execute_export_wizard_pipeline(&plan, &mut runner);

    assert!(execution.fatal);
    assert!(execution.stages.is_empty());
    assert!(runner.seen_stages.is_empty());
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("previous_pack and delta_pack")));
}

#[test]
fn export_wizard_pipeline_execution_stops_on_process_failure() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(2),
        stdout_lines: vec!["zircon_export stage=Validate profile=windows-release".to_string()],
        stderr_lines: vec!["validate failed".to_string()],
    });

    let execution = execute_export_wizard_pipeline(&plan, &mut runner);

    assert_eq!(execution.stages.len(), 1);
    assert!(execution.fatal);
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("Validate stderr: validate failed")));
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("exited with code 2")));
}
