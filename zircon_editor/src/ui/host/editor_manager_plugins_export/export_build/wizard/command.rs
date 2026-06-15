use zircon_runtime::plugin::ExportPipelineStage;

use super::{export_pipeline_stage_report_name, ExportWizardStageArtifactPath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPipelineStageCommand {
    pub stage: ExportPipelineStage,
    pub program: String,
    pub working_dir: Option<String>,
    pub args: Vec<String>,
    pub consumed_artifacts: Vec<ExportWizardStageArtifactPath>,
    pub produced_artifacts: Vec<ExportWizardStageArtifactPath>,
    pub expected_stdout_keys: Vec<&'static str>,
    pub missing_inputs: Vec<&'static str>,
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
            export_pipeline_stage_report_name(self.stage)
        )
    }
}
