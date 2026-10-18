use std::collections::BTreeMap;

use crate::{
    command_payload_descriptor, require_finite, validate_command_payload, ProtocolError,
    TELEMETRY_COMMAND_ID,
};

const LENGTH_PREFIX_BYTES: usize = 4;
const FIELD_COUNT_BYTES: usize = 2;
const NUMBER_BYTES: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryPayload {
    pub kind: String,
    pub data: BTreeMap<String, f64>,
}

impl TelemetryPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = telemetry_descriptor()?;
        validate_text_length(
            "TelemetryPayload.kind",
            self.kind.len(),
            descriptor.max_utf8_bytes,
        )?;
        validate_field_count(self.data.len(), descriptor.max_collection_entries)?;

        let mut bytes = Vec::new();
        append_text(&mut bytes, &self.kind);
        bytes.extend_from_slice(&(self.data.len() as u16).to_le_bytes());
        for (key, value) in self.data {
            validate_text_length(
                "TelemetryPayload.data key",
                key.len(),
                descriptor.max_utf8_bytes,
            )?;
            append_text(&mut bytes, &key);
            let value = canonical_finite_f64("TelemetryPayload.data value", value)?;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        validate_command_payload(TELEMETRY_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(TELEMETRY_COMMAND_ID, bytes)?;
        decode_payload(bytes)
    }
}

pub(crate) fn validate_telemetry_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_payload(bytes)?;
    Ok(())
}

fn decode_payload(bytes: &[u8]) -> Result<TelemetryPayload, ProtocolError> {
    let descriptor = telemetry_descriptor()?;
    let mut cursor = 0;
    let kind = read_text(
        bytes,
        &mut cursor,
        "TelemetryPayload.kind",
        descriptor.max_utf8_bytes,
    )?;
    let field_count = read_u16(bytes, &mut cursor, "TelemetryPayload.field_count")? as usize;
    validate_field_count(field_count, descriptor.max_collection_entries)?;

    let mut data = BTreeMap::new();
    for _ in 0..field_count {
        let key = read_text(
            bytes,
            &mut cursor,
            "TelemetryPayload.data key",
            descriptor.max_utf8_bytes,
        )?;
        let value = read_finite_f64(bytes, &mut cursor, "TelemetryPayload.data value")?;
        if data.insert(key.clone(), value).is_some() {
            return Err(ProtocolError::DuplicateCommandObjectKey { key });
        }
    }
    if cursor != bytes.len() {
        return Err(ProtocolError::TrailingPayload {
            remaining: bytes.len() - cursor,
        });
    }
    Ok(TelemetryPayload { kind, data })
}

fn telemetry_descriptor() -> Result<&'static crate::CommandPayloadDescriptor, ProtocolError> {
    command_payload_descriptor(TELEMETRY_COMMAND_ID).ok_or(
        ProtocolError::UnsupportedCommandPayload(TELEMETRY_COMMAND_ID),
    )
}

fn validate_text_length(
    context: &'static str,
    length: usize,
    maximum: usize,
) -> Result<(), ProtocolError> {
    if length > maximum || u32::try_from(length).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: length,
            maximum,
        });
    }
    Ok(())
}

fn validate_field_count(count: usize, maximum: usize) -> Result<(), ProtocolError> {
    if count > maximum || u16::try_from(count).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context: "TelemetryPayload.data",
            actual: count,
            maximum,
        });
    }
    Ok(())
}

fn append_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn read_text(
    bytes: &[u8],
    cursor: &mut usize,
    context: &'static str,
    maximum: usize,
) -> Result<String, ProtocolError> {
    let length = read_u32(bytes, cursor, context)? as usize;
    validate_text_length(context, length, maximum)?;
    let value = take(bytes, cursor, length, context)?;
    std::str::from_utf8(value)
        .map(str::to_owned)
        .map_err(|_| ProtocolError::InvalidUtf8 { context })
}

fn read_u16(bytes: &[u8], cursor: &mut usize, context: &'static str) -> Result<u16, ProtocolError> {
    Ok(u16::from_le_bytes(
        take(bytes, cursor, FIELD_COUNT_BYTES, context)?
            .try_into()
            .expect("fixed telemetry u16 slice"),
    ))
}

fn read_u32(bytes: &[u8], cursor: &mut usize, context: &'static str) -> Result<u32, ProtocolError> {
    Ok(u32::from_le_bytes(
        take(bytes, cursor, LENGTH_PREFIX_BYTES, context)?
            .try_into()
            .expect("fixed telemetry u32 slice"),
    ))
}

fn read_finite_f64(
    bytes: &[u8],
    cursor: &mut usize,
    context: &'static str,
) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        take(bytes, cursor, NUMBER_BYTES, context)?
            .try_into()
            .expect("fixed telemetry f64 slice"),
    );
    canonical_finite_f64(context, value)
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn take<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    count: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let remaining = bytes.len().saturating_sub(*cursor);
    let end = cursor
        .checked_add(count)
        .filter(|end| *end <= bytes.len())
        .ok_or(ProtocolError::TruncatedPayload {
            context,
            needed: count,
            remaining,
        })?;
    let value = &bytes[*cursor..end];
    *cursor = end;
    Ok(value)
}

#[cfg(test)]
#[path = "tests/telemetry_payload.rs"]
mod tests;
