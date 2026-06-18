use std::path::PathBuf;

use crate::ui::host::{export_wizard_compile_host_executable_path, ExportWizardPipelineOptions};
use crate::ui::workbench::project::project_root_path;
use zircon_runtime::plugin::ExportProfile;

use super::super::*;

const DEFAULT_SOURCE_ASSET_MANIFEST: &str = "assets/assets.json";

impl RetainedEditorHost {
    pub(super) fn export_wizard_options(
        &self,
        profile_name: &str,
    ) -> Result<ExportWizardPipelineOptions, String> {
        let project_path = self.runtime.editor_snapshot().project_path;
        let project_root = project_root_path(&project_path).map_err(|error| error.to_string())?;
        let manifest_path = project_root.join("zircon-project.toml");
        let output_root = self.effective_desktop_export_output_root(&project_root, profile_name);
        let profile = build_export_actions::desktop_export_profile(profile_name)
            .ok_or_else(|| format!("unknown desktop export profile `{profile_name}`"))?;
        let mut options = ExportWizardPipelineOptions::new(
            profile_name,
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
        options.host_executable = Some(export_wizard_default_host_executable(
            &options.out,
            &profile,
            options.target_dir.as_deref(),
        ));
        options.target_platform =
            Some(build_export_actions::export_platform_label(profile.target_platform).to_string());
        Ok(options)
    }
}

fn export_wizard_default_host_executable(
    out: &str,
    profile: &ExportProfile,
    target_dir: Option<&str>,
) -> String {
    export_wizard_compile_host_executable_path(out, profile, target_dir)
}

fn export_wizard_engine_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside the engine repository")
        .to_path_buf()
}
