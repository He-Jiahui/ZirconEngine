use std::path::PathBuf;

use crate::core::export::{ExportPresetStore, PlatformBundleLayout};
use crate::ui::host::{
    export_wizard_compile_host_executable_path, EditorExportBuildError, ExportWizardPipelineOptions,
};
use crate::ui::workbench::project::project_root_path;
use zircon_runtime::{
    asset::ProjectManifest, core::framework::platform::RuntimeTargetMode,
    core::framework::project::ExportProfile,
};
use zircon_runtime_interface::export::ExportTargetMode;

use super::super::*;

const DEFAULT_SOURCE_ASSET_MANIFEST: &str = "assets/assets.json";

impl RetainedEditorHost {
    pub(super) fn export_wizard_options(
        &self,
        preset_name: &str,
    ) -> Result<ExportWizardPipelineOptions, EditorExportBuildError> {
        let project_path = self.runtime.editor_snapshot().project_path;
        let project_root = project_root_path(&project_path)?;
        let manifest_path = project_root.join("zircon-project.toml");
        let store = ExportPresetStore::new(&project_root);
        let preset = store.load(preset_name)?;
        let preset_path = store.preset_path(preset_name)?;
        let output_root = self.effective_desktop_export_output_root(&project_root, preset_name);
        let manifest = ProjectManifest::load(&manifest_path).map_err(|source| {
            EditorExportBuildError::project_manifest(manifest_path.clone(), source)
        })?;
        let profile = manifest
            .export_profiles
            .into_iter()
            .find(|profile| profile.name == preset.profile_ref)
            .ok_or_else(|| EditorExportBuildError::unknown_profile(&preset.profile_ref))?;
        validate_preset_profile_target_mode(preset_name, preset.target_mode, &profile)?;
        let mut options = ExportWizardPipelineOptions::from_preset(
            preset_name,
            preset_path.display().to_string(),
            preset,
            manifest_path.display().to_string(),
            output_root.display().to_string(),
        );
        options.strategies = Some(profile.strategies.clone());
        options.repo_root = Some(export_wizard_engine_repo_root().display().to_string());
        options.source_asset_manifest = Some(
            output_root
                .join(DEFAULT_SOURCE_ASSET_MANIFEST)
                .display()
                .to_string(),
        );
        options.host_executable = Some(
            PlatformBundleLayout::expected(
                output_root
                    .join("stages")
                    .join("compile_host")
                    .join("staged"),
                options.preset.target_mode,
            )
            .launcher
            .display()
            .to_string(),
        );
        options.target_platform =
            Some(build_export_actions::export_platform_label(profile.target_platform).to_string());
        Ok(options)
    }
}

fn validate_preset_profile_target_mode(
    preset_name: &str,
    preset_mode: ExportTargetMode,
    profile: &ExportProfile,
) -> Result<(), EditorExportBuildError> {
    if matches!(
        (preset_mode, profile.target_mode),
        (
            ExportTargetMode::ClientRuntime,
            RuntimeTargetMode::ClientRuntime
        ) | (
            ExportTargetMode::ServerRuntime,
            RuntimeTargetMode::ServerRuntime
        )
    ) {
        return Ok(());
    }
    let profile_mode = match profile.target_mode {
        RuntimeTargetMode::ClientRuntime => "client_runtime",
        RuntimeTargetMode::ServerRuntime => "server_runtime",
        RuntimeTargetMode::EditorHost => "editor_host",
    };
    Err(EditorExportBuildError::PresetTargetModeMismatch {
        preset_name: preset_name.to_string(),
        profile_name: profile.name.clone(),
        preset_mode,
        profile_mode,
    })
}

pub(super) fn export_wizard_default_host_executable(
    out: &str,
    profile: &ExportProfile,
    target_dir: Option<&str>,
) -> String {
    export_wizard_compile_host_executable_path(out, profile, target_dir)
}

pub(super) fn export_wizard_engine_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside the engine repository")
        .to_path_buf()
}
