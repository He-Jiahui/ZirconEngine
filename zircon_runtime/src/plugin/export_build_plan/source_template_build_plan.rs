//! SourceTemplate 的可执行验证计划进入 Validate 报告，由 Python source-template 阶段核对命令与构建结果。
use serde::{Deserialize, Serialize};

use crate::core::framework::project::{ExportBuildMode, ExportPackagingStrategy};

use super::ExportBuildPlan;

const MANIFEST_PATH: &str = "Cargo.toml";
const TARGET_DIR: &str = "stages/source_template/target";
const BASE_COMMAND_ARGUMENT_COUNT: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Validate stage 交给 SourceTemplate 的构建声明；命令与结果由流水线独立核对。
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

/// 仅 SourceTemplate 有独立生成项目可编译；后续 Python 阶段负责执行及验证这份计划。
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
#[path = "tests/source_template_build_plan.rs"]
mod tests;
