use std::path::{Path, PathBuf};

use zircon_runtime::{
    plugin::{ExportBuildMode, ExportPipelineStage, ExportProfile, ExportTargetPlatform},
    RuntimeTargetMode,
};

use super::*;

#[test]
fn export_wizard_compile_host_path_feeds_platform_bundle_host_input() {
    let profile = desktop_windows_profile();
    let out = output_root();
    let mut options =
        ExportWizardPipelineOptions::new(profile.name.as_str(), "zircon-project.toml", out);
    options.source_asset_manifest = Some(
        PathBuf::from("target")
            .join("source-assets")
            .join("assets.json")
            .to_string_lossy()
            .into_owned(),
    );
    let host = export_wizard_compile_host_executable_path(
        &options.out,
        &profile,
        options.target_dir.as_deref(),
    );
    options.host_executable = Some(host.clone());

    let plan = export_wizard_pipeline_plan(options);

    assert!(plan.is_ready(), "{plan:?}");
    let compile_host = plan
        .command(ExportPipelineStage::CompileHost)
        .expect("CompileHost command should be planned");
    assert!(compile_host.expected_stdout_keys.contains(&"host"));

    let platform_bundle = plan
        .command(ExportPipelineStage::PlatformBundle)
        .expect("PlatformBundle command should be planned");
    assert_eq!(
        platform_bundle.argument_value("--host-executable"),
        Some(host.as_str())
    );
    assert!(platform_bundle
        .consumed_artifacts
        .iter()
        .any(|artifact| { artifact.key == "host" && artifact.path == host }));
    assert!(Path::new(&host).ends_with(expected_host_suffix("debug")));
}

#[test]
fn export_wizard_compile_host_path_respects_target_dir_override_and_build_mode() {
    let profile = desktop_windows_profile().with_build_mode(ExportBuildMode::Release);
    let target_dir = PathBuf::from("target").join("custom-compile-host-target");
    let target_dir_string = target_dir.to_string_lossy().into_owned();

    let host = export_wizard_compile_host_executable_path(
        output_root().as_str(),
        &profile,
        Some(target_dir_string.as_str()),
    );

    assert_eq!(
        host,
        target_dir
            .join("release")
            .join(format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX))
            .to_string_lossy()
            .into_owned()
    );
}

fn desktop_windows_profile() -> ExportProfile {
    ExportProfile::new(
        "desktop_windows",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
}

fn output_root() -> String {
    PathBuf::from("target")
        .join("zircon-export")
        .to_string_lossy()
        .into_owned()
}

fn expected_host_suffix(cargo_profile: &str) -> PathBuf {
    PathBuf::from("stages")
        .join("compile_host")
        .join("target")
        .join(cargo_profile)
        .join(format!("zircon_runtime{}", std::env::consts::EXE_SUFFIX))
}
