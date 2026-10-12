use std::ffi::OsString;
use std::path::PathBuf;
use zircon_runtime_interface::export::ExportStage;
use zircon_runtime_interface::export::{ExportPreset, ExportTargetMode};

use super::ExportWizardStageArtifactPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPipelineStageCommand {
    pub stage: ExportStage,
    pub program: String,
    pub working_dir: Option<String>,
    pub args: Vec<String>,
    pub consumed_artifacts: Vec<ExportWizardStageArtifactPath>,
    pub produced_artifacts: Vec<ExportWizardStageArtifactPath>,
    pub expected_stdout_keys: Vec<&'static str>,
    pub missing_inputs: Vec<&'static str>,
    pub core_projection: Option<ExportWizardCoreStageProjection>,
    pub native_program: Option<OsString>,
    pub native_args: Option<Vec<OsString>>,
    pub native_working_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportWizardCoreStageProjection {
    CompileHost {
        report_path: String,
        profile: String,
        host_path: String,
        preset: ExportPreset,
        repo_root: String,
        build_output_root: String,
        python: String,
        cargo: String,
        locked: bool,
        dry_run: bool,
    },
    PlatformBundle {
        build_output_root: String,
        target_mode: ExportTargetMode,
        dry_run: bool,
        preset: ExportPreset,
        repo_root: String,
        python: String,
        cargo: String,
        locked: bool,
        report_path: String,
    },
}

impl ExportWizardPipelineStageCommand {
    pub fn argv(&self) -> Vec<String> {
        let mut argv = Vec::with_capacity(self.args.len() + 1);
        argv.push(self.program.clone());
        argv.extend(self.args.iter().cloned());
        argv
    }

    pub fn argument_value(&self, option: &str) -> Option<&str> {
        self.args
            .windows(2)
            .find(|window| window[0] == option)
            .map(|window| window[1].as_str())
    }

    pub fn contains_flag(&self, flag: &str) -> bool {
        self.args.iter().any(|arg| arg == flag)
    }

    pub fn stdout_banner(&self, profile: &str) -> String {
        format!(
            "zircon_export stage={} profile={profile}",
            self.stage.report_name()
        )
    }
}
