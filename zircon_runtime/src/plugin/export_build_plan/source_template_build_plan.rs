use serde::{Deserialize, Serialize};

use crate::core::framework::project::{ExportBuildMode, ExportPackagingStrategy};

use super::ExportBuildPlan;

const MANIFEST_PATH: &str = "Cargo.toml";
const TARGET_DIR: &str = "stages/source_template/target";
const BASE_COMMAND_ARGUMENT_COUNT: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceTemplateBuildValidationPlan {
    pub manifest_path: String,
    pub target_dir: String,
    pub cargo_profile: String,
    pub release: bool,
    pub command: Vec<String>,
}

impl ExportBuildPlan {
    pub(crate) fn set_source_template_build_validation_plan(
        &mut self,
        plan: Option<SourceTemplateBuildValidationPlan>,
    ) {
        self.source_template_build = plan;
    }
}

pub(super) fn source_template_build_validation_plan(
    plan: &ExportBuildPlan,
) -> Option<SourceTemplateBuildValidationPlan> {
    if !plan
        .profile
        .uses_strategy(ExportPackagingStrategy::SourceTemplate)
    {
        return None;
    }

    let release = plan.profile.build_mode == ExportBuildMode::Release;
    let command = source_template_command(release);

    Some(SourceTemplateBuildValidationPlan {
        manifest_path: MANIFEST_PATH.to_string(),
        target_dir: TARGET_DIR.to_string(),
        cargo_profile: if release { "release" } else { "debug" }.to_string(),
        release,
        command,
    })
}

fn source_template_command(release: bool) -> Vec<String> {
    let mut command = Vec::with_capacity(BASE_COMMAND_ARGUMENT_COUNT + usize::from(release));
    command.extend([
        "cargo".to_string(),
        "build".to_string(),
        "--manifest-path".to_string(),
        MANIFEST_PATH.to_string(),
        "--target-dir".to_string(),
        TARGET_DIR.to_string(),
    ]);
    if release {
        command.push("--release".to_string());
    }
    command
}

#[cfg(test)]
mod tests {
    use super::source_template_command;

    #[test]
    fn preallocated_source_template_command_preserves_contract() {
        assert_eq!(
            source_template_command(true),
            vec![
                "cargo".to_string(),
                "build".to_string(),
                "--manifest-path".to_string(),
                "Cargo.toml".to_string(),
                "--target-dir".to_string(),
                "stages/source_template/target".to_string(),
                "--release".to_string(),
            ]
        );
    }
}
