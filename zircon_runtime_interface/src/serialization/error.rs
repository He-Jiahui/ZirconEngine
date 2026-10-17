use thiserror::Error;

use super::MigrateError;

/// Typed load failure for a versioned payload.
/// 版本化载荷读取的 wire 解析、schema 兼容性、迁移和目标类型解码错误。
///
/// 带 source 字段的变体保留底层错误链；其余变体提供结构化上下文或原因文本。
#[derive(Debug, Error)]
pub enum LoadError {
    /// 顶层文本预探测或旧格式 JSON 读取失败；source 保留 serde 错误，与严格封套解析失败区分。
    #[error("malformed versioned text: {source}")]
    MalformedText {
        #[source]
        source: serde_json::Error,
    },
    /// 文本声明了保留封套，但封套结构不符合线格式；不会降级为旧裸载荷。
    #[error("invalid versioned payload envelope: {source}")]
    InvalidEnvelope {
        #[source]
        source: serde_json::Error,
    },
    /// 常规文本读取拒绝无封套输入；schema-zero 回退须显式选择兼容入口。
    #[error("schema {schema_id} requires a versioned text envelope")]
    MissingTextEnvelope { schema_id: String },
    /// 文本文档字节长度超过固定输入上限，在封套和载荷解析前失败。
    /// found 统计完整文本输入字节（含封套）；仅 found > max 时产生此错误，等于上限继续解析。
    /// 该上限约束读取字节数，不衡量目标类型解码后全部内存分配。
    #[error("versioned text is too large: maximum {max} bytes, found {found}")]
    TextDocumentTooLarge { max: usize, found: usize },
    /// 二进制输入短于解析 wire 头所需的字节数。
    /// 仅表示固定 wire 前缀不完整；前缀完整后的封套头格式错误归入 MalformedBinary。
    #[error(
        "binary payload header is truncated: expected at least {expected} bytes, found {found}"
    )]
    BinaryHeaderTruncated { expected: usize, found: usize },
    /// 二进制头部的魔数与当前格式标识不匹配。
    #[error("binary payload magic is invalid: found {found:?}")]
    BinaryMagicMismatch { found: [u8; 8] },
    /// 头部声明了不支持的 wire 格式版本；它与载荷的 schema 版本不同。
    #[error("binary wire version {found} is unsupported; supported version is {supported}")]
    UnsupportedBinaryWireVersion { found: u16, supported: u16 },
    /// 二进制载荷主体超过读取上限。
    /// found 统计固定前缀后的封套 body（含头部和载荷）；仅 found > max 时拒绝。
    #[error("binary payload body is too large: maximum {max} bytes, found {found}")]
    BinaryPayloadTooLarge { max: usize, found: usize },
    /// 二进制位置编码字节不符合线格式结构，并保留底层解码错误。
    #[error("malformed versioned binary payload: {source}")]
    MalformedBinary {
        #[source]
        source: super::PositionalDecodeError,
    },
    /// 二进制值域载荷无效，保留 decoder 提供的原因文本。
    #[error("invalid binary value-domain payload: {reason}")]
    InvalidBinaryPayload { reason: String },
    /// 封套中的 schema 身份与目标类型声明的身份不同。
    #[error("expected schema {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },
    /// 输入 schema 版本高于目标类型的当前版本，loader 不尝试解码。
    #[error("schema {schema_id} version {found} is newer than supported version {supported}")]
    FutureVersion {
        schema_id: String,
        found: u32,
        supported: u32,
    },
    /// schema/版本检查及必要迁移后，目标类型反序列化失败；source 保留 serde 错误。
    #[error("schema {schema_id} version {schema_version} payload decode failed: {source}")]
    PayloadDecode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
    /// 迁移表校验或迁移回调失败，内层原因由 MigrateError 保留。
    #[error(transparent)]
    Migration(#[from] MigrateError),
}
