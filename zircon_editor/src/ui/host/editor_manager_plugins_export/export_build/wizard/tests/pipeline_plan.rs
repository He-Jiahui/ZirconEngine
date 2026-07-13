use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime_interface::export::ExportStage;

use super::super::*;
use super::support::*;

#[test]
fn export_wizard_pipeline_plan_builds_stage_commands_in_cli_order() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.repo_root = Some("E:\\Git\\ZirconEngine".to_string());
    options.source_asset_manifest = Some("D:\\assets\\cooked-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options.offline = true;
    options.dry_run = true;

    let plan = export_wizard_pipeline_plan(options);

    assert!(plan.is_ready(), "{:?}", plan);
    assert_eq!(
        plan.stages
            .iter()
            .map(|command| command.stage)
            .collect::<Vec<_>>(),
        ExportStage::ALL.to_vec()
    );

    let validate = plan
        .command(ExportStage::Validate)
        .expect("validate command");
    assert_eq!(validate.program, "python");
    assert_eq!(validate.argument_value("--stage"), Some("validate"));
    assert_eq!(
        validate.argument_value("--profile"),
        Some("windows-release")
    );
    assert_eq!(
        validate.argument_value("--preset"),
        Some("export/windows-release.zpreset")
    );
    assert_eq!(
        validate.argument_value("--project"),
        Some("zircon-project.toml")
    );
    assert!(validate.contains_flag("--offline"));
    assert!(validate.contains_flag("--dry-run"));
    assert_eq!(
        validate
            .argument_value("--stage")
            .expect("stage argument should exist")
            .parse::<ExportStage>()
            .ok(),
        Some(ExportStage::Validate)
    );

    let native_dynamic = plan
        .command(ExportStage::NativeDynamic)
        .expect("native dynamic command");
    assert_eq!(
        native_dynamic.argument_value("--stage"),
        Some("native_dynamic")
    );
    assert_eq!(
        native_dynamic.argument_value("--validate-report"),
        Some("D:\\zircon-export\\stages\\validate\\report.json")
    );
    assert_eq!(
        "native_dynamic".parse::<ExportStage>().ok(),
        Some(ExportStage::NativeDynamic)
    );
    assert_eq!(
        "NativeDynamic".parse::<ExportStage>().ok(),
        Some(ExportStage::NativeDynamic)
    );
}

#[test]
fn export_wizard_pipeline_plan_selects_stages_from_packaging_strategies() {
    let library_embed = export_wizard_pipeline_plan(
        ready_export_options().with_strategies([ExportPackagingStrategy::LibraryEmbed]),
    );
    assert_eq!(
        stage_sequence(&library_embed),
        vec![
            ExportStage::Validate,
            ExportStage::CompileHost,
            ExportStage::CookAssets,
            ExportStage::Pack,
            ExportStage::PlatformBundle,
            ExportStage::Report,
        ]
    );
    assert!(library_embed.command(ExportStage::SourceTemplate).is_none());
    assert!(library_embed.command(ExportStage::NativeDynamic).is_none());

    let source_template = export_wizard_pipeline_plan(
        ready_export_options().with_strategies([ExportPackagingStrategy::SourceTemplate]),
    );
    assert_eq!(
        stage_sequence(&source_template),
        vec![
            ExportStage::Validate,
            ExportStage::SourceTemplate,
            ExportStage::Report,
        ]
    );

    let native_dynamic = export_wizard_pipeline_plan(
        ready_export_options().with_strategies([ExportPackagingStrategy::NativeDynamic]),
    );
    assert_eq!(
        stage_sequence(&native_dynamic),
        vec![
            ExportStage::Validate,
            ExportStage::NativeDynamic,
            ExportStage::CompileHost,
            ExportStage::CookAssets,
            ExportStage::Pack,
            ExportStage::PlatformBundle,
            ExportStage::Report,
        ]
    );

    let combined = export_wizard_pipeline_plan(ready_export_options().with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::LibraryEmbed,
    ]));
    assert_eq!(stage_sequence(&combined), ExportStage::ALL.to_vec());
}

#[test]
fn export_wizard_pipeline_plan_threads_stage_artifact_inputs() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.pack_file = Some("D:\\zircon-export\\custom\\game.zrpack".to_string());
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());
    options.delta_pack = Some("D:\\zircon-export\\custom\\game.zrpd".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options.template_dir = Some(
        "tools\\zircon_export\\export-templates\\windows-x86_64-library_embed-debug".to_string(),
    );
    options.determinism_check = true;

    let plan = export_wizard_pipeline_plan(options);

    let native_dynamic = plan
        .command(ExportStage::NativeDynamic)
        .expect("native dynamic command");
    assert_eq!(
        native_dynamic.argument_value("--validate-report"),
        Some("D:\\zircon-export\\stages\\validate\\report.json")
    );
    assert!(native_dynamic.produced_artifacts.iter().any(|artifact| {
        artifact.key == "plugins_dir" && artifact.path.ends_with("stages\\native_dynamic\\plugins")
    }));
    assert!(native_dynamic.produced_artifacts.iter().any(|artifact| {
        artifact.key == "loader_manifest"
            && artifact
                .path
                .ends_with("stages\\native_dynamic\\plugins\\native_plugins.toml")
    }));

    let cook_assets = plan
        .command(ExportStage::CookAssets)
        .expect("cook assets command");
    assert_eq!(
        cook_assets.argument_value("--asset-manifest"),
        Some("D:\\assets\\source-assets.json")
    );
    assert!(cook_assets.produced_artifacts.iter().any(|artifact| {
        artifact.key == "cooked_asset_manifest"
            && artifact.path.ends_with("stages\\cook_assets\\assets.json")
    }));

    let pack = plan.command(ExportStage::Pack).expect("pack command");
    assert_eq!(
        pack.argument_value("--asset-manifest"),
        Some("D:\\zircon-export\\stages\\cook_assets\\assets.json")
    );
    assert_eq!(
        pack.argument_value("--pack-file"),
        Some("D:\\zircon-export\\custom\\game.zrpack")
    );
    assert_eq!(
        pack.argument_value("--previous-pack"),
        Some("D:\\old\\game.zrpack")
    );
    assert_eq!(
        pack.argument_value("--delta-pack"),
        Some("D:\\zircon-export\\custom\\game.zrpd")
    );
    assert!(pack.contains_flag("--determinism-check"));

    let platform_bundle = plan
        .command(ExportStage::PlatformBundle)
        .expect("platform bundle command");
    assert_eq!(
        platform_bundle.argument_value("--pack-file"),
        Some("D:\\zircon-export\\custom\\game.zrpack")
    );
    assert_eq!(
        platform_bundle.argument_value("--host-executable"),
        Some("D:\\zircon-export\\host\\ZirconRuntime.exe")
    );
    assert_eq!(
        platform_bundle.argument_value("--template-dir"),
        Some("tools\\zircon_export\\export-templates\\windows-x86_64-library_embed-debug")
    );
}

#[test]
fn export_wizard_pipeline_plan_reports_missing_execution_inputs() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());

    let plan = export_wizard_pipeline_plan(options);

    assert!(!plan.is_ready());
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("previous_pack and delta_pack")));
    assert!(plan
        .command(ExportStage::CookAssets)
        .expect("cook assets command")
        .missing_inputs
        .contains(&"source_asset_manifest"));
    assert!(plan
        .command(ExportStage::PlatformBundle)
        .expect("platform bundle command")
        .missing_inputs
        .contains(&"host_executable"));
    assert!(plan
        .command(ExportStage::Pack)
        .expect("pack command")
        .missing_inputs
        .contains(&"previous_pack+delta_pack"));
}

#[test]
fn export_wizard_pipeline_banners_drive_progress_parser() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut progress = ExportWizardProgressState::new();

    for command in &plan.stages {
        progress.push_stdout_line(&command.stdout_banner(&plan.profile));
    }

    assert_eq!(progress.current_stage(), Some(ExportStage::Report));
    assert_eq!(
        progress
            .snapshot(ExportStage::Report)
            .expect("report snapshot")
            .profile
            .as_deref(),
        Some("windows-release")
    );
}
