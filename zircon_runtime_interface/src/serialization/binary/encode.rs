use serde_json::Value;

use crate::serialization::text::canonical::canonicalize_value;
use crate::serialization::{PayloadHeader, PositionalEncodeError, WriteError};

use super::envelope::BinaryEnvelope;
use super::value::BinaryValue;
use super::wire::{append_prefix, options, BINARY_PREFIX_LEN, MAX_BINARY_BODY_BYTES};

/// 将 writer 已序列化的 JSON 值编码为带 v1 前缀的二进制载荷。
///
/// 写入路径先规范化对象键顺序，再由 `BinaryValue::try_from` 检查二进制值域；封套 body
/// 受 [`MAX_BINARY_BODY_BYTES`] 限制，前缀不计入该限额。
///
/// # 参数
/// - `header`：随封套写入的 schema 标识和版本。
/// - `payload`：由版本化 writer 从领域值序列化得到的 JSON 值。
///
/// # 返回值
/// 成功时返回固定前缀和封套 body 拼接后的完整字节序列。
///
/// # 错误
/// payload 不符合二进制值域时返回 `InvalidBinaryPayload`，body 超限时返回
/// `BinaryPayloadTooLarge`，其余编码失败返回带 schema/version 上下文的 `BinaryEncode`。
pub(in crate::serialization) fn encode_binary_payload(
    header: PayloadHeader,
    payload: Value,
) -> Result<Vec<u8>, WriteError> {
    let schema_id = header.schema_id.as_str().to_string();
    let schema_version = header.schema_version;
    // 先按文本值域规范化对象键顺序，使相同逻辑值产生稳定的二进制节点序列。
    let value = BinaryValue::try_from(canonicalize_value(payload)).map_err(|source| {
        WriteError::InvalidBinaryPayload {
            schema_id: schema_id.clone(),
            schema_version,
            reason: source.to_string(),
        }
    })?;
    match encode_binary_value(header, value) {
        Ok(bytes) => Ok(bytes),
        Err(PositionalEncodeError::SizeLimit { .. }) => Err(WriteError::BinaryPayloadTooLarge {
            schema_id,
            schema_version,
            max: MAX_BINARY_BODY_BYTES,
        }),
        Err(source) => Err(WriteError::BinaryEncode {
            schema_id,
            schema_version,
            source,
        }),
    }
}

/// 将给定的 schema 头和扁平值载荷序列化为封套 body，并在其前追加 wire 前缀。
///
/// 本层只序列化并限制 body 字节数，不检查节点值域。生产入口
/// [`encode_binary_payload`] 先执行受限的 JSON 转换；畸形输入测试则刻意跳过该转换，
/// 用本函数封装非法节点以验证读取器的拒绝边界。编码选项由 [`super::wire::options`]
/// 固定，body 限额不含前缀。
pub(in crate::serialization) fn encode_binary_value(
    header: PayloadHeader,
    payload: BinaryValue,
) -> Result<Vec<u8>, PositionalEncodeError> {
    let envelope = BinaryEnvelope { header, payload };
    let body = options().encode_bounded(&envelope, MAX_BINARY_BODY_BYTES as u64)?;
    let mut bytes = Vec::with_capacity(BINARY_PREFIX_LEN + body.len());
    append_prefix(&mut bytes);
    bytes.extend_from_slice(&body);
    Ok(bytes)
}
