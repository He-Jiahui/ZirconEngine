use semver::VersionReq;

use super::directional_range::classify_incompatible_requirement;
use super::{
    ProjectEngineCompatibility, ProjectEngineCompatibilityDisposition,
    ProjectEngineCompatibilityError, ProjectEngineVersion,
};

/// Evaluates the manifest's declared engine range without loading any project-derived code.
/// 在加载项目派生代码前评估清单的引擎版本要求；无要求视为兼容，无法判定方向时保留不兼容结果。
///
/// # Parameters
/// - `requirement`：项目清单中的 semver 要求文本；`None` 表示清单未声明引擎版本要求。
/// - `running_engine`：宿主为本次预检选定的引擎版本，整个评估都以此值为准。
///
/// # Returns
/// 返回保留原要求文本、所选引擎版本和判定结果的 [`ProjectEngineCompatibility`] 快照。
///
/// # Errors
/// 传入 `Some` 的要求无法解析为 [`semver::VersionReq`] 时返回 [`ProjectEngineCompatibilityError::InvalidRequirement`]。
///
/// # 约束
/// `Compatible` 只代表 semver 要求匹配；项目身份、迁移状态、provider 能力和会话准入由调用端分别检查。
///
/// # 参考
/// Editor 在 `zircon_editor/src/core/project/project_preflight/preflight_receipt.rs` 的
/// `ProjectPreflightReceipt::evaluate_engine_compatibility` 中传入清单字段；
/// `zircon_runtime_interface/src/project/tests/engine_compatibility.rs` 覆盖结果快照和方向分类。
pub fn assess_project_engine_compatibility(
    requirement: Option<&str>,
    running_engine: &ProjectEngineVersion,
) -> Result<ProjectEngineCompatibility, ProjectEngineCompatibilityError> {
    let Some(requirement) = requirement else {
        return Ok(ProjectEngineCompatibility::new(
            None,
            running_engine.clone(),
            ProjectEngineCompatibilityDisposition::Compatible,
        ));
    };
    let parsed = VersionReq::parse(requirement).map_err(|source| {
        ProjectEngineCompatibilityError::InvalidRequirement {
            requirement: requirement.to_string(),
            source,
        }
    })?;
    let disposition = if parsed.matches(running_engine.as_semver()) {
        ProjectEngineCompatibilityDisposition::Compatible
    } else {
        classify_incompatible_requirement(&parsed, running_engine.as_semver())
    };
    Ok(ProjectEngineCompatibility::new(
        Some(requirement.to_string()),
        running_engine.clone(),
        disposition,
    ))
}
