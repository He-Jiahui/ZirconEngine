use std::path::{Path, PathBuf};

use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime_interface::export::ExportStage;

use super::*;

#[test]
fn export_wizard_report_command_consumes_source_template_report() {
    let plan = export_wizard_pipeline_plan(ready_options().with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::LibraryEmbed,
    ]));
    let report = plan
        .command(ExportStage::Report)
        .expect("Report command should be planned");

    assert!(report.consumed_artifacts.iter().any(|artifact| {
        artifact.key == "report"
            && Path::new(&artifact.path).ends_with(
                PathBuf::from("stages")
                    .join("source_template")
                    .join("report.json"),
            )
    }));
    assert!(report.consumed_artifacts.iter().any(|artifact| {
        artifact.key == "report"
            && Path::new(&artifact.path).ends_with(
                PathBuf::from("stages")
                    .join("native_dynamic")
                    .join("report.json"),
            )
    }));
    assert_eq!(
        report
            .consumed_artifacts
            .iter()
            .filter(|artifact| artifact.key == "report")
            .count(),
        7
    );
}

#[test]
fn export_wizard_report_command_skips_unplanned_strategy_reports() {
    let plan = export_wizard_pipeline_plan(
        ready_options().with_strategies([ExportPackagingStrategy::LibraryEmbed]),
    );
    let report = plan
        .command(ExportStage::Report)
        .expect("Report command should be planned");

    assert!(report.consumed_artifacts.iter().all(|artifact| {
        !Path::new(&artifact.path).ends_with(
            PathBuf::from("stages")
                .join("source_template")
                .join("report.json"),
        ) && !Path::new(&artifact.path).ends_with(
            PathBuf::from("stages")
                .join("native_dynamic")
                .join("report.json"),
        )
    }));
    assert_eq!(
        report
            .consumed_artifacts
            .iter()
            .filter(|artifact| artifact.key == "report")
            .count(),
        5
    );
}

fn ready_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\export\\host\\ZirconRuntime.exe".to_string());
    options
}
