use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::plugin::{ExportBuildMode, ExportPackagingStrategy};

use super::ExportBuildPlan;

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

    let manifest_path = Path::new("Cargo.toml").display().to_string();
    let target_dir = Path::new("stages")
        .join("source_template")
        .join("target")
        .display()
        .to_string()
        .replace('\\', "/");
    let release = plan.profile.build_mode == ExportBuildMode::Release;
    let mut command = vec![
        "cargo".to_string(),
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.clone(),
        "--target-dir".to_string(),
        target_dir.clone(),
    ];
    if release {
        command.push("--release".to_string());
    }

    Some(SourceTemplateBuildValidationPlan {
        manifest_path,
        target_dir,
        cargo_profile: if release { "release" } else { "debug" }.to_string(),
        release,
        command,
    })
}
