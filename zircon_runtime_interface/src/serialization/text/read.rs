//! 通过借用的 `RawValue` 识别版本封套，供加载层选择当前版本解码或迁移路径。
//! 对象封套的预探测仍会由 `serde_json` 扫描并验证 JSON 语法，只避免提前物化业务 payload。

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::serialization::PayloadHeader;

/// A text envelope borrows its payload so current-version readers can decode T directly.
/// `payload` 的借用与输入字节同寿命；加载器同步解码后只返回拥有所有权的 `Loaded<T>`。
pub(in crate::serialization) struct BorrowedTextEnvelope<'a> {
    /// 已反序列化的封套元数据，随后由加载层校验 schema 与版本。
    pub(in crate::serialization) header: PayloadHeader,
    /// 保留输入中的原始 JSON 片段，供后续直接解码为 `T` 或转换为迁移值。
    pub(in crate::serialization) payload: &'a RawValue,
}

/// 封套识别结果；未声明完整版本头的输入交由加载调用方按策略处理。
pub(in crate::serialization) enum TextInput<'a> {
    /// 已识别并严格解析封套，payload 仍借用原始输入。
    Envelope(BorrowedTextEnvelope<'a>),
    /// 当前输入没有完整版本封套签名，可由调用方按旧格式策略接纳或拒绝。
    Unversioned,
}

/// 区分顶层文档预探测错误与已提取 `$zircon` 候选的解析错误。
pub(in crate::serialization) enum TextReadError {
    /// 顶层 JSON 预探测失败，包括语法错误或顶层保留键重复等反序列化错误。
    Malformed(serde_json::Error),
    /// `$zircon` 候选在头部探测或严格封套解析中遇到字段、类型或结构错误。
    InvalidEnvelope(serde_json::Error),
}

/// 只探测顶层保留键，并把其值保留为原始片段以延后封套判定。
#[derive(Deserialize)]
struct EnvelopeProbe<'a> {
    #[serde(rename = "$zircon", default, borrow)]
    envelope: Option<&'a RawValue>,
}

/// 预探测封套中是否存在 `header`；缺失时保留为 `None`。
#[derive(Deserialize)]
struct EnvelopeHeaderProbe<'a> {
    #[serde(default, borrow)]
    header: Option<&'a RawValue>,
}

/// 仅检查版本头两个签名键是否出现；字段值校验留给完整封套解析。
#[derive(Deserialize)]
struct HeaderSignatureProbe<'a> {
    #[serde(default, borrow)]
    schema_id: Option<&'a RawValue>,
    #[serde(default, borrow)]
    schema_version: Option<&'a RawValue>,
}

/// 完整签名命中后使用的严格顶层结构，拒绝封套之外的未知字段。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTextDocument<'a> {
    #[serde(rename = "$zircon", borrow)]
    envelope: &'a RawValue,
}

/// 完整签名命中后使用的严格封套结构；payload 仍保留为借用的原始 JSON。
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTextEnvelope<'a> {
    header: PayloadHeader,
    #[serde(borrow)]
    payload: &'a RawValue,
}

/// Identifies the reserved envelope without materializing an unversioned payload `Value`.
/// 对象输入的预探测仍由 `serde_json` 扫描整份 JSON；`RawValue` 只延后 payload 的领域类型解码。
/// 完整版本头签名会触发严格封套解析，结构错误不会回退为旧格式；顶层探测与封套解析错误分别报告。
///
/// # 使用时机与生命周期
/// `load_text_versioned` 在校验 schema 与版本前调用此函数。返回封套的 payload 借用 `bytes`，
/// 调用方须在当前版本解码或迁移完成前保留输入。
///
/// # 返回值与错误
/// 无完整版本头签名时返回 `Unversioned`，是否按旧格式读取由调用方策略决定。顶层文档预探测失败
/// （例如 JSON 语法错误）为 `Malformed`；已提取的 `$zircon` 候选解析失败为 `InvalidEnvelope`。
pub(in crate::serialization) fn inspect_text(bytes: &[u8]) -> Result<TextInput<'_>, TextReadError> {
    if first_non_whitespace(bytes) != Some(b'{') {
        return Ok(TextInput::Unversioned);
    }

    let probe =
        serde_json::from_slice::<EnvelopeProbe<'_>>(bytes).map_err(TextReadError::Malformed)?;
    let Some(envelope) = probe.envelope else {
        return Ok(TextInput::Unversioned);
    };
    // 仅完整声明 schema_id 与 schema_version 才占用保留 envelope；同名业务字段仍可走 v0 迁移。
    if !claims_version_header(envelope).map_err(TextReadError::InvalidEnvelope)? {
        return Ok(TextInput::Unversioned);
    }
    let document = serde_json::from_slice::<RawTextDocument<'_>>(bytes)
        .map_err(TextReadError::InvalidEnvelope)?;
    let envelope = serde_json::from_str::<RawTextEnvelope<'_>>(document.envelope.get())
        .map_err(TextReadError::InvalidEnvelope)?;
    Ok(TextInput::Envelope(BorrowedTextEnvelope {
        header: envelope.header,
        payload: envelope.payload,
    }))
}

/// 按 `schema_id` 与 `schema_version` 两个键是否存在判定封套签名，不在此阶段校验其值。
fn claims_version_header(envelope: &RawValue) -> Result<bool, serde_json::Error> {
    if first_non_whitespace(envelope.get().as_bytes()) != Some(b'{') {
        return Ok(false);
    }
    let probe = serde_json::from_str::<EnvelopeHeaderProbe<'_>>(envelope.get())?;
    let Some(header) = probe.header else {
        return Ok(false);
    };
    if first_non_whitespace(header.get().as_bytes()) != Some(b'{') {
        return Ok(false);
    }
    let signature = serde_json::from_str::<HeaderSignatureProbe<'_>>(header.get())?;
    Ok(signature.schema_id.is_some() && signature.schema_version.is_some())
}

/// 跳过前导 ASCII 空白以检查首个字节；这一步不代替后续的 JSON 语法解析。
fn first_non_whitespace(bytes: &[u8]) -> Option<u8> {
    bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}
