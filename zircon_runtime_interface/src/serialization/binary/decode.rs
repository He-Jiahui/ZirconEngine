use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::serialization::{LoadError, PayloadHeader};

use super::wire::{body_after_valid_prefix, options};

/// 从完整 wire 输入解析前缀和封套头，并返回余下 payload 的借用切片。
///
/// 此阶段验证 magic、wire 版本、body 限额及头部自身的反序列化约束；目标 schema 和受支持
/// 的 schema 版本由上层 loader 在调用 payload 解码前检查。返回的 payload 尚未解释。
///
/// # 返回值
/// 头部和 body 中紧随头部的字节切片；切片生命周期受 `bytes` 限制。
pub(in crate::serialization) fn decode_binary_header(
    bytes: &[u8],
) -> Result<(PayloadHeader, &[u8]), LoadError> {
    let body = body_after_valid_prefix(bytes)?;
    // 仅解头时允许后续 payload 字节；完整 payload 解码仍拒绝尾随数据。
    let (header, payload_offset) = options()
        .decode_prefix_bounded(body, body.len() as u64)
        .map_err(|source| LoadError::MalformedBinary { source })?;
    Ok((header, &body[payload_offset..]))
}

/// 将不含固定前缀的完整 payload body 解码为 JSON 值。
///
/// 该入口用于需要把旧版本值送入迁移流程的 loader；完整解码会拒绝尾随字节，并在转换时
/// 再检查 `BinaryValue` 的值域限制。
pub(in crate::serialization) fn decode_binary_payload(body: &[u8]) -> Result<Value, LoadError> {
    let payload: super::value::BinaryValue = options()
        .decode(body)
        .map_err(|source| LoadError::MalformedBinary { source })?;
    Value::try_from(payload).map_err(|source: super::value::BinaryValueError| {
        LoadError::InvalidBinaryPayload {
            reason: source.to_string(),
        }
    })
}

/// 将当前版本的 payload body 直接解码为目标类型，不经 JSON 中间值。
///
/// 调用方须已依据同一封套头验证目标 schema 和 schema 版本。这里仍完整解码并校验扁平
/// `BinaryValue` 流，再由 serde 解码到 `T`；输入只应包含 payload body。
/// `T: DeserializeOwned` 保证返回值不借用该 body，调用方可在解码完成后释放输入切片。
pub(in crate::serialization) fn decode_binary_current<T>(
    body: &[u8],
) -> Result<T, super::value::DirectBinaryDecodeError>
where
    T: DeserializeOwned,
{
    let payload: super::value::BinaryValue = options()
        .decode(body)
        .map_err(super::value::DirectBinaryDecodeError::Malformed)?;
    super::value::decode_binary_value_direct(payload)
}
