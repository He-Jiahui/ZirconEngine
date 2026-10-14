use thiserror::Error;

use crate::project::ProjectNameError;
use crate::serialization::MigrateError;

/// Typed failure produced by the lightweight project-manifest reader.
///
/// 该类型覆盖摘要解析、输入限额、摘要字段约束和共享迁移失败；它不是 Runtime 完整清单校验错误。
///
/// # 使用时机与生命周期
/// 字节或字符串摘要入口、公开值域迁移入口在无法继续解析或迁移时返回此错误。
///
/// # 约束
/// InvalidEngineVersionReq 表示要求文本无法解析，不表示已经与某个运行引擎比较。
///
/// # 参考
/// 参考 [`crate::project::ProjectManifestSummary`] 的解析入口；Runtime 完整读取还会继续校验专属字段。
///
#[derive(Debug, Error)]
pub enum ProjectManifestSummaryError {
    /// 字节切片入口无法把原始清单解码为 UTF-8；字符串入口不会产生此变体。
    #[error("project manifest is not UTF-8: {source}")]
    InvalidUtf8 {
        #[source]
        source: std::str::Utf8Error,
    },
    /// UTF-8 输入未通过 TOML 语法解析。
    #[error("project manifest TOML is invalid: {source}")]
    InvalidToml {
        #[source]
        source: toml::de::Error,
    },
    /// TOML 结构已读取，但转换或反序列化为摘要输入形状失败。
    #[error("project manifest has an invalid field shape: {source}")]
    InvalidShape {
        #[source]
        source: serde_json::Error,
    },
    /// 原始输入字节数大于共享上限；max 和 found 均按字节计。
    #[error("project manifest exceeds {max} bytes (found {found})")]
    DocumentTooLarge { max: usize, found: usize },
    /// 已解析的 TOML 表或数组层级超过容器深度上限。
    #[error("project manifest TOML nesting exceeds depth {max} (found {found})")]
    TomlNestingTooDeep { max: usize, found: usize },
    /// 整棵 TOML 值树中的表项累计数超过上限，而非单张表的大小。
    #[error("project manifest TOML tables exceed {max} cumulative entries (found {found})")]
    TooManyTomlTableEntries { max: usize, found: usize },
    /// 整棵 TOML 值树中的数组元素累计数超过上限，而非单个数组的大小。
    #[error("project manifest TOML arrays exceed {max} cumulative items (found {found})")]
    TooManyTomlArrayItems { max: usize, found: usize },
    /// 结构可读，但字段间关系或摘要层语义约束不成立。
    #[error("project manifest has an invalid value: {message}")]
    InvalidValue { message: String },
    /// 项目名未通过共享的跨平台名称校验。
    #[error("project manifest project name is invalid: {source}")]
    InvalidProjectName {
        #[source]
        source: ProjectNameError,
    },
    /// asset_roots 项数超过共享解析器允许的数量。
    #[error("project manifest declares {found} asset roots; maximum is {max}")]
    TooManyAssetRoots { max: usize, found: usize },
    /// 两个已解析的 RelPath 根相等，不能作为不同资产根同时保留。
    #[error("project manifest declares duplicate normalized asset root {root}")]
    DuplicateAssetRoot { root: String },
    /// 一个资产根是另一个根的路径组件祖先；仅字符串前缀相同不足以构成此错误。
    #[error("project manifest asset root {ancestor} contains nested root {descendant}")]
    OverlappingAssetRoots {
        ancestor: String,
        descendant: String,
    },
    /// Some(engine_version_req) 无法按 semver::VersionReq 语法解析。
    #[error("project manifest engine_version_req {value:?} is invalid: {source}")]
    InvalidEngineVersionReq {
        value: String,
        #[source]
        source: semver::Error,
    },
    /// format_version 缺失以外的值不是可表示为 u32 的无符号整数。
    #[error("project manifest format_version must be an unsigned 32-bit integer")]
    InvalidFormatVersion,
    /// 输入版本高于本模块迁移链目标，拒绝将未来格式降解读取。
    #[error("project manifest version {found} is newer than supported version {supported}")]
    FutureVersion { found: u32, supported: u32 },
    /// 版本链校验或迁移步骤失败；thiserror 的 from 属性保留底层 MigrateError。
    #[error(transparent)]
    Migration(#[from] MigrateError),
}
