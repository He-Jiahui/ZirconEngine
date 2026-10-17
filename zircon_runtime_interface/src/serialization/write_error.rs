use std::io;

use thiserror::Error;

/// Typed failure while streaming canonical JSON to an arbitrary byte sink.
#[derive(Debug, Error)]
/// 规范文本写入的分阶段失败；调用方可区分载荷拒绝、预算超限和 sink I/O。
pub enum CanonicalTextWriteError {
    #[error("canonical text contains non-finite float {value}")]
    /// 载荷遍历发现非有限浮点值。
    NonFinite { value: f64 },
    /// 序列化器自定义校验失败或载荷形状不受支持。
    #[error("canonical text payload validation failed: {reason}")]
    PayloadValidation { reason: String },
    /// 载荷编码器返回非校验类序列化错误。
    #[error("canonical text payload encode failed: {reason}")]
    PayloadEncode { reason: String },
    #[error("canonical text exceeds the {max}-byte wire limit (found at least {found} bytes)")]
    /// 规范文本超过本次输出字节上限（固定文档上限或调用方预算）；`found` 是已知至少达到的字节数。
    OutputTooLarge { max: usize, found: usize },
    #[error("canonical text {resource} exceeds the limit {max} (found {found})")]
    /// 超过规范写入器的某项资源限额；`found` 与 `max` 使用该资源的计量单位。
    ResourceLimitExceeded {
        resource: &'static str,
        max: usize,
        found: usize,
    },
    #[error("canonical text {operation} failed: {source}")]
    /// 向 sink 写入时指定操作失败；此前成功写入的字节不会由此错误撤销。
    Io {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
}

impl serde::ser::Error for CanonicalTextWriteError {
    fn custom<T>(message: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::PayloadValidation {
            reason: message.to_string(),
        }
    }
}

/// Typed failure while encoding a current-version payload.
#[derive(Debug, Error)]
/// 版本化编码的统一错误；变体保留文本/二进制阶段及 schema/version 上下文。
pub enum WriteError {
    #[error("schema {schema_id} version {schema_version} contains non-finite float {value}")]
    /// 文本或二进制载荷遍历遇到非有限浮点。
    NonFiniteFloat {
        schema_id: String,
        schema_version: u32,
        value: f64,
    },
    /// 载荷自定义验证或支持域检查失败。
    #[error("schema {schema_id} version {schema_version} payload validation failed: {reason}")]
    PayloadValidation {
        schema_id: String,
        schema_version: u32,
        reason: String,
    },
    /// 文本载荷编码或二进制分支转换为 JSON 值时发生序列化错误。
    #[error("schema {schema_id} version {schema_version} payload encode failed: {source}")]
    PayloadEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "schema {schema_id} version {schema_version} text document exceeds the {max}-byte wire limit (found at least {found} bytes)"
    )]
    /// 版本化文本输出超过固定 wire 字节上限。
    TextDocumentTooLarge {
        schema_id: String,
        schema_version: u32,
        max: usize,
        found: usize,
    },
    #[error(
        "schema {schema_id} version {schema_version} canonical text {resource} exceeds the limit {max} (found {found})"
    )]
    /// 版本化规范文本在命名资源上超过限制；计量单位随 `resource` 定义。
    TextResourceLimitExceeded {
        schema_id: String,
        schema_version: u32,
        resource: &'static str,
        max: usize,
        found: usize,
    },
    #[error("schema {schema_id} version {schema_version} binary encode failed: {source}")]
    /// 位置式二进制编码器拒绝载荷或编码失败。
    BinaryEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: super::PositionalEncodeError,
    },
    #[error(
        "schema {schema_id} version {schema_version} binary body exceeds the {max}-byte wire limit"
    )]
    /// 二进制载荷体超过 wire 格式允许的最大字节数。
    BinaryPayloadTooLarge {
        schema_id: String,
        schema_version: u32,
        max: usize,
    },
    #[error("schema {schema_id} version {schema_version} binary value is invalid: {reason}")]
    /// 二进制封套编码前的载荷值不符合该格式的可表示约束。
    InvalidBinaryPayload {
        schema_id: String,
        schema_version: u32,
        reason: String,
    },
    #[error("schema {schema_id} version {schema_version} text encode failed: {source}")]
    /// 版本化规范文本转换为 `String` 时的编码或 UTF-8 错误。
    TextEncode {
        schema_id: String,
        schema_version: u32,
        #[source]
        source: serde_json::Error,
    },
    #[error("schema {schema_id} version {schema_version} text sink {operation} failed: {source}")]
    /// 流式文本写入的 sink 操作失败；错误保留具体操作名和底层 I/O 来源。
    TextWrite {
        schema_id: String,
        schema_version: u32,
        operation: &'static str,
        #[source]
        source: io::Error,
    },
}
