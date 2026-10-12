use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::serialization::{LoadError, PayloadHeader};

use super::wire::{body_after_valid_prefix, options};

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
