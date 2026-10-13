//! 读取带版本的文本和二进制载荷，并在解码前验证身份与版本边界。
//!
//! 普通入口要求显式 envelope；只有明确选择 [`load_versioned_legacy_schema_zero`] 的调用方
//! 才会把无 envelope 的文本作为 schema 0 送入迁移链。当前版本直接解码到目标类型，旧版本
//! 才经 JSON 值迁移，以便兼容历史形状。

use std::io;

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::binary::{
    decode_binary_current, decode_binary_header, decode_binary_payload, DirectBinaryDecodeError,
};
use super::text::read::{inspect_text, TextInput, TextReadError};
use super::text::wire::MAX_TEXT_DOCUMENT_BYTES;
use super::{Format, LoadError, Loaded, VersionedSchema};

/// Loads a versioned payload and rejects text that lacks the reserved `$zircon` envelope.
///
/// # 使用时机与生命周期
/// 用于持久文档或传输载荷的常规读取。调用期间借用输入字节；成功值由返回的 [`Loaded`]
/// 持有，不再依赖输入切片。
///
/// # 参数
/// - `bytes`：完整的文本文档或二进制版本封装。
/// - `format`：选择与写入方一致的 wire 格式。
///
/// # 返回值
/// 返回当前类型及可选迁移来源；格式、schema、版本、迁移或 payload 校验失败时返回
/// [`LoadError`]。
///
/// # 约束
/// 文本必须含 `$zircon` envelope；二进制必须含有效前缀和头部。该入口不会推断无头文本为
/// schema 0。这里完成 wire/schema/version 检查和目标类型解码；类型解码后的领域不变量由具体
/// 调用方继续校验。
///
/// # 参考
/// [`VersionedSchema`] 定义目标身份和版本；仅兼容历史裸文本的调用方应显式选择
/// [`load_versioned_legacy_schema_zero`]。
pub fn load_versioned<T>(bytes: &[u8], format: Format) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    match format {
        Format::Text => load_text_versioned(bytes, UnversionedTextPolicy::Reject),
        Format::Binary => load_binary_versioned(bytes),
    }
}

/// Explicit alias for profiles whose public contract requires a text envelope.
///
/// # 使用时机
/// 当 API 或配置档案需要明确表达“文本必须带版本封装”时使用；其格式和错误行为与
/// [`load_versioned`] 相同。
///
/// # 参数
/// - `bytes`：完整封装字节。
/// - `format`：输入格式；选择二进制时仍按二进制头部契约读取。
///
/// # 返回值
/// 返回经验证的当前类型和迁移来源，或相应 [`LoadError`]。
pub fn load_versioned_envelope<T>(bytes: &[u8], format: Format) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    load_versioned(bytes, format)
}

/// Loads a historical schema-zero text payload when no envelope is present.
///
/// Callers must opt into this only for schemas with an owned v0 migration policy.
/// 无 envelope 的回退仅适用于文本；二进制始终需要有效的线格式前缀和版本。
///
/// # 使用时机与生命周期
/// 仅在该数据所有者明确承诺读取旧版裸文本时调用。`bytes` 只在本次读取期间借用，返回值
/// 与输入解耦。
///
/// # 参数
/// - `bytes`：带 envelope 的当前/旧版文本，或始终带头部的二进制数据。
/// - `format`：文本允许受控的 schema-zero 兼容路径；二进制仍必须包含线格式封装。
///
/// # 返回值
/// 返回迁移后的当前类型；缺少迁移步骤、类型解码失败或 wire 头部不合法时返回 [`LoadError`]。
/// 若裸文本从版本 0 成功升级，结果会记录 `migrated_from: Some(0)`。
///
/// # 约束
/// 不要将此入口用于没有负责迁移旧形状的 schema；普通读取优先使用 [`load_versioned`]。
pub fn load_versioned_legacy_schema_zero<T>(
    bytes: &[u8],
    format: Format,
) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    match format {
        Format::Text => load_text_versioned(bytes, UnversionedTextPolicy::AcceptSchemaZero),
        Format::Binary => load_binary_versioned(bytes),
    }
}

/// 裸文本在当前调用点的处理策略；仅显式兼容入口选择接受 schema 0。
#[derive(Clone, Copy)]
enum UnversionedTextPolicy {
    /// 将没有 envelope 的文本作为 schema 0 交给目标类型的迁移链。
    AcceptSchemaZero,
    /// 拒绝裸文本，并报告目标 schema 身份。
    Reject,
}

fn load_text_versioned<T>(
    bytes: &[u8],
    unversioned_policy: UnversionedTextPolicy,
) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    // 在调用格式解析器前限制整个文档大小，避免超预算输入进入 envelope/payload 解析。
    if bytes.len() > MAX_TEXT_DOCUMENT_BYTES {
        return Err(LoadError::TextDocumentTooLarge {
            max: MAX_TEXT_DOCUMENT_BYTES,
            found: bytes.len(),
        });
    }
    match inspect_text(bytes).map_err(text_read_error)? {
        // 是否允许 schema-zero 由公开入口显式传入；不能因 envelope 解析失败而降级。
        TextInput::Unversioned => match unversioned_policy {
            UnversionedTextPolicy::AcceptSchemaZero => load_schema_zero_text(bytes),
            UnversionedTextPolicy::Reject => Err(LoadError::MissingTextEnvelope {
                schema_id: T::SCHEMA.as_str().to_string(),
            }),
        },
        TextInput::Envelope(envelope) => {
            // 先校验头部身份和版本，再读取 payload，保证错 schema/未来版本不会触发目标解码。
            validate_schema::<T>(&envelope.header.schema_id)?;
            validate_source_version::<T>(envelope.header.schema_version)?;
            // 当前文本版本从借用的原始片段直接解析目标类型，旧版本才进入 JSON 值域迁移。
            if envelope.header.schema_version == T::VERSION {
                // 当前 schema 直接从借用的 payload 片段构造 T；完整迁移表仍先作为 schema 不变量校验。
                validate_migration_chain::<T>()?;
                let value =
                    serde_json::from_str::<T>(envelope.payload.get()).map_err(|source| {
                        LoadError::PayloadDecode {
                            schema_id: T::SCHEMA.as_str().to_string(),
                            schema_version: T::VERSION,
                            source,
                        }
                    })?;
                Ok(Loaded {
                    value,
                    migrated_from: None,
                })
            } else {
                // 旧格式进入可变换的 JSON 值域，迁移完成后再反序列化为当前类型。
                let payload = serde_json::from_str(envelope.payload.get())
                    .map_err(|source| LoadError::InvalidEnvelope { source })?;
                load_migrated_value(payload, envelope.header.schema_version)
            }
        }
    }
}

fn load_schema_zero_text<T>(bytes: &[u8]) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    // 裸文本没有可供 sniff 的 schema 身份，按调用者选择的兼容策略直接视为版本 0。
    let payload =
        serde_json::from_slice(bytes).map_err(|source| LoadError::MalformedText { source })?;
    load_migrated_value(payload, 0)
}

fn load_binary_versioned<T>(bytes: &[u8]) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    // 先拆出二进制头部并验证身份/版本；payload 只在头部可接受后解码。
    let (header, payload_body) = decode_binary_header(bytes)?;
    validate_schema::<T>(&header.schema_id)?;
    validate_source_version::<T>(header.schema_version)?;
    if header.schema_version == T::VERSION {
        // 当前二进制 schema 也校验完整迁移链，再走不经 serde_json::Value 的 typed decode。
        validate_migration_chain::<T>()?;
        let value = decode_binary_current(payload_body)
            .map_err(|error| binary_current_decode_error::<T>(error))?;
        return Ok(Loaded {
            value,
            migrated_from: None,
        });
    }
    // 旧版本才构造成迁移值域，以复用文本路径的 schema 演进规则。
    let payload = decode_binary_payload(payload_body)?;
    load_migrated_value(payload, header.schema_version)
}

fn load_migrated_value<T>(payload: Value, source_version: u32) -> Result<Loaded<T>, LoadError>
where
    T: VersionedSchema + DeserializeOwned + 'static,
{
    // 复核来源版本后记录迁移起点；无升级时保持 None，再执行整条前向链并构造目标类型。
    validate_source_version::<T>(source_version)?;
    let migrated_from = (source_version < T::VERSION).then_some(source_version);
    let payload = T::migrations().migrate_value(&T::SCHEMA, payload, source_version, T::VERSION)?;
    let value = serde_json::from_value(payload).map_err(|source| LoadError::PayloadDecode {
        schema_id: T::SCHEMA.as_str().to_string(),
        schema_version: T::VERSION,
        source,
    })?;
    Ok(Loaded {
        value,
        migrated_from,
    })
}

fn validate_migration_chain<T>() -> Result<(), LoadError>
where
    T: VersionedSchema + 'static,
{
    // 当前载荷也必须携带完整迁移链，避免坏 schema 仅因读到最新版而潜伏。
    T::migrations()
        .validate(&T::SCHEMA, T::VERSION)
        .map_err(LoadError::Migration)
}

fn text_read_error(error: TextReadError) -> LoadError {
    // 保留 envelope 错误与普通 JSON 语法错误的区分，供调用方定位 wire 层失败。
    match error {
        TextReadError::Malformed(source) => LoadError::MalformedText { source },
        TextReadError::InvalidEnvelope(source) => LoadError::InvalidEnvelope { source },
    }
}

fn binary_current_decode_error<T>(error: DirectBinaryDecodeError) -> LoadError
where
    T: VersionedSchema,
{
    // 将二进制 decoder 的 wire、节点值和目标类型错误映射到稳定的公开加载错误类别。
    match error {
        DirectBinaryDecodeError::Malformed(source) => LoadError::MalformedBinary { source },
        DirectBinaryDecodeError::Invalid(source) => LoadError::InvalidBinaryPayload {
            reason: source.to_string(),
        },
        DirectBinaryDecodeError::Payload(reason) => LoadError::PayloadDecode {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            source: serde_json::Error::io(io::Error::new(io::ErrorKind::InvalidData, reason)),
        },
    }
}

fn validate_source_version<T>(source_version: u32) -> Result<(), LoadError>
where
    T: VersionedSchema,
{
    // 未来数据不能按当前类型解码或静默退回迁移；向调用方报告双方版本以供显式升级策略处理。
    if source_version <= T::VERSION {
        return Ok(());
    }
    Err(LoadError::FutureVersion {
        schema_id: T::SCHEMA.as_str().to_string(),
        found: source_version,
        supported: T::VERSION,
    })
}

fn validate_schema<T>(found: &super::SchemaId) -> Result<(), LoadError>
where
    T: VersionedSchema,
{
    // schema 身份不匹配时在访问 payload 前失败，避免把另一种文档解释为目标类型。
    if found == &T::SCHEMA {
        return Ok(());
    }
    Err(LoadError::SchemaMismatch {
        expected: T::SCHEMA.as_str().to_string(),
        found: found.as_str().to_string(),
    })
}
