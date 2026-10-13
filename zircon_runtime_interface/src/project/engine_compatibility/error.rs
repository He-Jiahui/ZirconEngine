use thiserror::Error;

/// Reports an engine requirement that cannot be safely evaluated during preflight.
/// 项目清单引擎版本要求无法解析时返回的评估错误。
///
/// # 使用时机
/// [`crate::project::assess_project_engine_compatibility`] 传入 `Some` 的清单要求无法解析时返回此错误；方向无法证明仍是判定值，不会转成解析错误。
///
/// # 参考
/// `zircon_runtime_interface/src/project/tests/engine_compatibility.rs` 区分无效要求错误与不确定兼容结论。
#[derive(Debug, Error)]
pub enum ProjectEngineCompatibilityError {
    #[error("project engine-version requirement {requirement:?} is invalid: {source}")]
    /// 清单要求文本不符合 semver 区间语法。
    InvalidRequirement {
        /// 评估时收到的原始要求，供调用端诊断。
        requirement: String,
        /// semver 解析器提供的原因，并作为错误源暴露。
        #[source]
        source: semver::Error,
    },
}
