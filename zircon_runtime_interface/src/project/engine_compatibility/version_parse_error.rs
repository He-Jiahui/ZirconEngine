use thiserror::Error;

/// Reports a running-engine value that is not canonical semantic-version input.
/// 报告无法解析为 semver 的宿主引擎版本输入，供预检失败诊断。
///
/// # 使用时机
/// [`crate::project::ProjectEngineVersion::parse`] 在宿主版本文本无法解析时返回此错误；反序列化也经由同一解析入口。
///
/// # 参考
/// `zircon_runtime_interface/src/project/tests/engine_compatibility.rs` 锁定有效版本的 wire 往返；无效值由调用端作为预检失败处理。
#[derive(Debug, Error)]
pub enum ProjectEngineVersionParseError {
    #[error("project engine version {value:?} is invalid: {source}")]
    /// 保存原始版本输入和 semver 解析失败原因。
    Invalid {
        /// 宿主或 wire 数据提供的原始版本文本。
        value: String,
        /// semver 解析器提供的原因，并作为错误源暴露。
        #[source]
        source: semver::Error,
    },
}
