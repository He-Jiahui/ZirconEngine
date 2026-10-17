use thiserror::Error;

use crate::project::{ProjectManifestSummaryError, ProjectNameError, RelPathError};

/// Typed failure while validating and rendering an embedded project template pack.
/// 模板包校验、清单改写或渲染阶段产生的结构化失败。
#[derive(Debug, Error)]
pub enum ProjectTemplatePackError {
    /// 新项目名称未通过共享的项目名称校验。
    #[error("project template project name is invalid: {source}")]
    InvalidProjectName {
        #[source]
        source: ProjectNameError,
    },
    /// 嵌入条目路径不符合 `RelPath` 的词法相对路径约束。
    #[error("template entry path is invalid: {source}")]
    InvalidEntryPath {
        #[from]
        #[source]
        source: RelPathError,
    },
    /// 多个嵌入条目规范化后占用了相同的项目相对路径。
    #[error("project template contains duplicate entry path {path}")]
    DuplicateEntryPath { path: String },
    /// 一项的文件路径同时是另一项的目录祖先，导致条目布局无法逐项落盘。
    #[error("project template entry path {path} conflicts with file ancestor {ancestor}")]
    EntryPathConflictsWithFile { path: String, ancestor: String },
    /// 嵌入模板条目中没有必需的 `zircon-project.toml`。
    #[error("project template does not contain zircon-project.toml")]
    MissingManifest,
    /// 处理模板清单字节时，输入不能解码为 UTF-8。
    #[error("project template manifest is not UTF-8: {source}")]
    ManifestUtf8 {
        #[source]
        source: std::str::Utf8Error,
    },
    /// 模板清单不能解析为 TOML 值。
    #[error("project template manifest TOML is invalid: {source}")]
    ManifestToml {
        #[source]
        source: toml::de::Error,
    },
    /// 身份字段改写后的 TOML 清单无法重新编码。
    #[error("project template manifest could not be encoded: {source}")]
    ManifestEncode {
        #[source]
        source: toml::ser::Error,
    },
    /// 清单的 provider 选择缺失、重复，或不满足目标要求。
    #[error("project template manifest does not satisfy its descriptor: {reason}")]
    ManifestRequirements { reason: String },
    /// 改写后的清单未通过共享的项目摘要解析。
    #[error("rendered project template manifest is invalid: {source}")]
    ManifestSummary {
        #[from]
        #[source]
        source: ProjectManifestSummaryError,
    },
}
