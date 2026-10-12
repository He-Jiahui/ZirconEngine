const PYTHON_EXECUTABLE: &str = "python";

use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime_interface::export::{ExportPreset, ExportTargetMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPipelineOptions {
    pub preset_name: String,
    pub preset_path: String,
    pub preset: ExportPreset,
    pub python: String,
    pub project: String,
    pub out: String,
    pub strategies: Option<Vec<ExportPackagingStrategy>>,
    pub repo_root: Option<String>,
    pub cargo: Option<String>,
    pub validator: Option<String>,
    pub packer: Option<String>,
    pub source_asset_manifest: Option<String>,
    pub pack_file: Option<String>,
    pub previous_pack: Option<String>,
    pub delta_pack: Option<String>,
    pub host_executable: Option<String>,
    pub template_dir: Option<String>,
    pub engine_version: Option<String>,
    pub target_platform: Option<String>,
    pub target_dir: Option<String>,
    pub offline: bool,
    pub no_locked: bool,
    pub pretty: bool,
    pub dry_run: bool,
    pub source_template_build: bool,
    pub determinism_check: bool,
}

impl ExportWizardPipelineOptions {
    pub fn from_preset(
        preset_name: impl Into<String>,
        preset_path: impl Into<String>,
        preset: ExportPreset,
        project: impl Into<String>,
        out: impl Into<String>,
    ) -> Self {
        Self {
            preset_name: preset_name.into(),
            preset_path: preset_path.into(),
            preset,
            python: PYTHON_EXECUTABLE.to_string(),
            project: project.into(),
            out: out.into(),
            strategies: None,
            repo_root: None,
            cargo: None,
            validator: None,
            packer: None,
            source_asset_manifest: None,
            pack_file: None,
            previous_pack: None,
            delta_pack: None,
            host_executable: None,
            template_dir: None,
            engine_version: None,
            target_platform: None,
            target_dir: None,
            offline: false,
            no_locked: false,
            pretty: false,
            dry_run: false,
            source_template_build: false,
            determinism_check: false,
        }
    }

    #[cfg(test)]
    pub fn for_test_profile(
        profile: impl Into<String>,
        project: impl Into<String>,
        out: impl Into<String>,
    ) -> Self {
        let profile = profile.into();
        let preset_name = profile.clone();
        let preset_path = format!("export/{preset_name}.zpreset");
        let preset = ExportPreset::new(profile, ExportTargetMode::ClientRuntime);
        Self::from_preset(preset_name, preset_path, preset, project, out)
    }

    pub fn with_strategies(
        mut self,
        strategies: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.strategies = Some(strategies.into_iter().collect());
        self
    }
}
