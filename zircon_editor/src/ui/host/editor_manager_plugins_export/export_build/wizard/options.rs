const PYTHON_EXECUTABLE: &str = "python";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPipelineOptions {
    pub python: String,
    pub profile: String,
    pub project: String,
    pub out: String,
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
    pub fn new(
        profile: impl Into<String>,
        project: impl Into<String>,
        out: impl Into<String>,
    ) -> Self {
        Self {
            python: PYTHON_EXECUTABLE.to_string(),
            profile: profile.into(),
            project: project.into(),
            out: out.into(),
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
}
