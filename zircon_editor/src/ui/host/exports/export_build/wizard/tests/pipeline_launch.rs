use zircon_runtime_interface::export::ExportStage;

use super::*;

#[test]
fn export_wizard_pipeline_commands_use_repo_root_as_working_dir() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside the engine repository")
        .to_string_lossy()
        .into_owned();
    let mut options = ready_options();
    options.repo_root = Some(repo_root.clone());

    let plan = export_wizard_pipeline_plan(options);

    assert!(plan
        .stages
        .iter()
        .all(|command| command.working_dir.as_deref() == Some(repo_root.as_str())));
    let validate = plan
        .command(ExportStage::Validate)
        .expect("validate command should exist");
    assert_eq!(
        validate.argument_value("--repo-root"),
        Some(repo_root.as_str())
    );
}

#[test]
fn export_wizard_pipeline_uses_explicit_compile_host_root_without_repo_root_override() {
    let plan = export_wizard_pipeline_plan(ready_options());

    assert!(plan
        .stages
        .iter()
        .filter(|command| command.stage != ExportStage::CompileHost)
        .all(|command| command.working_dir.is_none()));
    assert_eq!(
        plan.command(ExportStage::CompileHost)
            .expect("compile host command should exist")
            .working_dir
            .as_deref(),
        Some(".")
    );
    let validate = plan
        .command(ExportStage::Validate)
        .expect("validate command should exist");
    assert_eq!(validate.argument_value("--repo-root"), None);
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
