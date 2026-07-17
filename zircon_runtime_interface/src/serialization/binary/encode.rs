use bincode::Options;
use serde_json::Value;

use crate::serialization::text::canonical::canonicalize_value;
use crate::serialization::{PayloadHeader, WriteError};

use super::envelope::BinaryEnvelope;
use super::value::BinaryValue;
use super::wire::{append_prefix, options, BINARY_PREFIX_LEN, MAX_BINARY_BODY_BYTES};

pub(in crate::serialization) fn encode_binary_payload(
    header: PayloadHeader,
    payload: Value,
) -> Result<Vec<u8>, WriteError> {
    let schema_id = header.schema_id.as_str().to_string();
    let schema_version = header.schema_version;
    let value = BinaryValue::try_from(canonicalize_value(payload)).map_err(|source| {
        WriteError::InvalidBinaryPayload {
            schema_id: schema_id.clone(),
            schema_version,
            reason: source.to_string(),
        }
    })?;
    match encode_binary_value(header, value) {
        Ok(bytes) => Ok(bytes),
        Err(source) if matches!(source.as_ref(), bincode::ErrorKind::SizeLimit) => {
            Err(WriteError::BinaryPayloadTooLarge {
                schema_id,
                schema_version,
                max: MAX_BINARY_BODY_BYTES,
            })
        }
        Err(source) => Err(WriteError::BinaryEncode {
            schema_id,
            schema_version,
            source,
        }),
    }
}

pub(in crate::serialization) fn encode_binary_value(
    header: PayloadHeader,
    payload: BinaryValue,
) -> bincode::Result<Vec<u8>> {
    let envelope = BinaryEnvelope { header, payload };
    let body = options()
        .with_limit(MAX_BINARY_BODY_BYTES as u64)
        .serialize(&envelope)?;
    let mut bytes = Vec::with_capacity(BINARY_PREFIX_LEN + body.len());
    append_prefix(&mut bytes);
    bytes.extend_from_slice(&body);
    Ok(bytes)
}
