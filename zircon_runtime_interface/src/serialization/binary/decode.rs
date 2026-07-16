use std::io::Cursor;

use bincode::Options;
use serde_json::Value;

use crate::serialization::{LoadError, PayloadHeader};

use super::wire::{body_after_valid_prefix, options};

pub(in crate::serialization) fn decode_binary_header(
    bytes: &[u8],
) -> Result<(PayloadHeader, &[u8]), LoadError> {
    let body = body_after_valid_prefix(bytes)?;
    let mut cursor = Cursor::new(body);
    let header = options()
        .allow_trailing_bytes()
        .with_limit(body.len() as u64)
        .deserialize_from(&mut cursor)
        .map_err(|source| LoadError::MalformedBinary { source })?;
    let payload_offset = cursor.position() as usize;
    Ok((header, &body[payload_offset..]))
}

pub(in crate::serialization) fn decode_binary_payload(body: &[u8]) -> Result<Value, LoadError> {
    let payload: super::value::BinaryValue = options()
        .with_limit(body.len() as u64)
        .deserialize(body)
        .map_err(|source| LoadError::MalformedBinary { source })?;
    Value::try_from(payload).map_err(|source: super::value::BinaryValueError| {
        LoadError::InvalidBinaryPayload {
            reason: source.to_string(),
        }
    })
}
