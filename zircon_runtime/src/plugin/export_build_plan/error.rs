//! 建计划阶段的 profile 查找错误经编辑器和验证 CLI 分别转换为构建报告或 fatal 校验报告。
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
/// 计划尚未生成时的入口错误；调用方不能把它当作可继续 materialize 的诊断计划。
pub enum ExportBuildPlanError {
    #[error("missing export profile {profile_name}")]
    MissingProfile { profile_name: String },
}
