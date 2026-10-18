use serde::Deserialize;
use woc_protocol::{event_stream_digest, fnv1a_bytes, DecodeLimits, FixedTickInput, WorldSnapshot};
use zr_vm_rust_binding::{Value, ValueKind};

use super::exports::binding_error;
use crate::VmTickError;

pub(super) fn lower_bytes(bytes: &[u8]) -> Result<Value, VmTickError> {
    check_length(bytes.len())?;
    let mut array = Value::new_array().map_err(binding_error)?;
    for byte in bytes {
        let value = Value::new_int(i64::from(*byte)).map_err(binding_error)?;
        array.array_push(&value).map_err(binding_error)?;
    }
    Ok(array)
}

pub(super) fn lift_bytes(value: &Value) -> Result<Vec<u8>, VmTickError> {
    lift_bytes_with_limit(
        value,
        DecodeLimits::default().max_payload_bytes,
        "WOC byte payload exceeds the protocol limit",
    )
}

/// Presentation crosses the VM boundary as bytes; scalar string reads use a
/// fixed binding buffer and cannot carry a complete client projection.
pub(super) fn lift_presentation_bytes(value: &Value) -> Result<Vec<u8>, VmTickError> {
    lift_bytes_with_limit(
        value,
        crate::MAX_CLIENT_PRESENTATION_BYTES,
        "WOC presentation payload exceeds the client projection limit",
    )
}

fn lift_bytes_with_limit(
    value: &Value,
    maximum: usize,
    limit_message: &'static str,
) -> Result<Vec<u8>, VmTickError> {
    if value.kind() != ValueKind::Array {
        return Err(VmTickError::Transport(
            "WOC byte export must return Array<uint>".into(),
        ));
    }
    let length = value.array_len().map_err(binding_error)?;
    if length > maximum {
        return Err(VmTickError::Transport(limit_message.into()));
    }
    let mut bytes = Vec::with_capacity(length);
    for index in 0..length {
        let value = value.array_get(index).map_err(binding_error)?;
        let integer = value.as_int().map_err(binding_error)?;
        bytes.push(u8::try_from(integer).map_err(|_| {
            VmTickError::Transport(format!("WOC byte at index {index} is outside 0..255"))
        })?);
    }
    Ok(bytes)
}

fn check_length(length: usize) -> Result<(), VmTickError> {
    if length > DecodeLimits::default().max_payload_bytes {
        return Err(VmTickError::Transport(
            "WOC byte payload exceeds the protocol limit".into(),
        ));
    }
    Ok(())
}

pub(super) fn require_success(value: Value) -> Result<(), VmTickError> {
    match value.as_int().map_err(binding_error)? {
        0 => Ok(()),
        status => Err(VmTickError::Trap(format!(
            "WOC lifecycle export returned {status}"
        ))),
    }
}

pub(super) fn validate_world_state(state: &[u8]) -> Result<(), VmTickError> {
    check_length(state.len())?;
    if state.is_empty() {
        return Ok(());
    }
    if state.get(..4) != Some(b"WOS2".as_slice())
        || state.get(4..6)
            != Some(
                woc_protocol::WORLD_STATE_SCHEMA_VERSION
                    .to_le_bytes()
                    .as_slice(),
            )
    {
        return Err(VmTickError::Transport(format!(
            "world state must use {}",
            woc_protocol::WORLD_STATE_FORMAT
        )));
    }
    // The package's restoreState owns the inner WOS schema and validates it.
    Ok(())
}

pub(super) fn decode_tick_input(bytes: &[u8]) -> Result<FixedTickInput, VmTickError> {
    let input = FixedTickInput::decode_payload(bytes)
        .map_err(|error| VmTickError::Transport(format!("invalid fixed tick input: {error:?}")))?;
    if !input.wall_time_forbidden
        || fnv1a_bytes(&input.committed_state) != input.committed_state_digest
    {
        return Err(VmTickError::Transport(
            "fixed tick input violates the canonical state contract".into(),
        ));
    }
    validate_world_state(&input.committed_state)?;
    Ok(input)
}

pub(super) fn validate_tick_output(bytes: &[u8], expected_tick: u64) -> Result<(), VmTickError> {
    let output = WorldSnapshot::decode_payload(bytes)
        .map_err(|error| VmTickError::Transport(format!("invalid fixed tick output: {error:?}")))?;
    if output.tick != expected_tick
        || output.state_digest != fnv1a_bytes(&output.state)
        || output.event_digest != event_stream_digest(&output.events)
        || output.state.is_empty()
    {
        return Err(VmTickError::Transport(
            "fixed tick output violates the canonical snapshot contract".into(),
        ));
    }
    validate_world_state(&output.state)
}

#[derive(Deserialize)]
struct PackageSchema {
    schema_version: u32,
    types: Vec<serde_json::Value>,
    package: String,
    contract: String,
    command_catalog: String,
    command_payload_schema: String,
    world_state: String,
    simulation_hz: u32,
    presentation_hz: u32,
}

pub(super) fn validate_schema(schema: &str) -> Result<(), VmTickError> {
    let schema: PackageSchema = serde_json::from_str(schema)
        .map_err(|error| VmTickError::Transport(format!("invalid WOC stateSchema: {error}")))?;
    if schema.schema_version != 3
        || !schema.types.is_empty()
        || schema.package != "woc_game"
        || schema.contract != woc_protocol::SCHEMA_FINGERPRINT_HEX
        || schema.command_catalog != woc_protocol::COMMAND_CATALOG_SHA256
        || schema.command_payload_schema != woc_protocol::COMMAND_PAYLOAD_SCHEMA_SHA256
        || schema.world_state != woc_protocol::WORLD_STATE_FORMAT
        || schema.simulation_hz != woc_protocol::SIMULATION_HZ
        || schema.presentation_hz != woc_protocol::PRESENTATION_HZ
    {
        return Err(VmTickError::Limited(
            "WOC package schema differs from the native protocol authority".into(),
        ));
    }
    Ok(())
}
