use std::path::{Path, PathBuf};

use zircon_runtime::plugin::ExportPipelineStage;

use super::*;

#[test]
fn export_wizard_report_command_consumes_source_template_report() {
    let plan = export_wizard_pipeline_plan(ready_options());
    let report = plan
        .command(ExportPipelineStage::Report)
        .expect("Report command should be planned");

    assert!(report.consumed_artifacts.iter().any(|artifact| {
        artifact.key == "report"
            && Path::new(&artifact.path).ends_with(
                PathBuf::from("stages")
                    .join("source_template")
                    .join("report.json"),
            )
    }));
    assert_eq!(
        report
            .consumed_artifacts
            .iter()
            .filter(|artifact| artifact.key == "report")
            .count(),
        6
    );
}

fn ready_options() -> ExportWizardPipelineOptions {
    let mut options =
        ExportWizardPipelineOptions::new("windows-release", "zircon-project.toml", "D:\\export");
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\export\\host\\ZirconRuntime.exe".to_string());
    options
}
