use std::collections::BTreeSet;

use crate::{
    command_payload_descriptor, validate_command_payload, CommandPayloadDescriptor, ProtocolError,
    SET_TOWN_FOCUS_COMMAND_ID,
};

const LENGTH_PREFIX_BYTES: usize = 4;
const ENTRY_COUNT_BYTES: usize = 2;
const POINTS_BYTES: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TownFocusAllocationEntry {
    pub component: String,
    pub points: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TownFocusCommandPayload {
    pub allocation: Vec<TownFocusAllocationEntry>,
}

impl TownFocusCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = town_focus_descriptor()?;
        validate_entry_count(self.allocation.len(), descriptor.max_collection_entries)?;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.allocation.len() as u16).to_le_bytes());
        let mut components = BTreeSet::new();
        for entry in &self.allocation {
            if !components.insert(entry.component.as_str()) {
                return Err(ProtocolError::DuplicateCommandObjectKey {
                    key: entry.component.clone(),
                });
            }
            validate_component_length(entry.component.len(), descriptor.max_utf8_bytes)?;
            bytes.extend_from_slice(&(entry.component.len() as u32).to_le_bytes());
            bytes.extend_from_slice(entry.component.as_bytes());
            bytes.extend_from_slice(&entry.points.to_le_bytes());
        }
        validate_command_payload(SET_TOWN_FOCUS_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(SET_TOWN_FOCUS_COMMAND_ID, bytes)?;
        decode_payload(bytes)
    }
}

pub(crate) fn validate_town_focus_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_payload(bytes)?;
    Ok(())
}

fn decode_payload(bytes: &[u8]) -> Result<TownFocusCommandPayload, ProtocolError> {
    let descriptor = town_focus_descriptor()?;
    let mut cursor = 0;
    let entry_count = read_u16(bytes, &mut cursor, "TownFocusCommandPayload.entry_count")? as usize;
    validate_entry_count(entry_count, descriptor.max_collection_entries)?;

    let mut allocation = Vec::with_capacity(entry_count);
    let mut components = BTreeSet::new();
    for _ in 0..entry_count {
        let component = read_component(bytes, &mut cursor, descriptor.max_utf8_bytes)?;
        let points = read_i32(bytes, &mut cursor, "TownFocusCommandPayload.points")?;
        if !components.insert(component.clone()) {
            return Err(ProtocolError::DuplicateCommandObjectKey { key: component });
        }
        allocation.push(TownFocusAllocationEntry { component, points });
    }
    if cursor != bytes.len() {
        return Err(ProtocolError::TrailingPayload {
            remaining: bytes.len() - cursor,
        });
    }
    Ok(TownFocusCommandPayload { allocation })
}

fn town_focus_descriptor() -> Result<&'static CommandPayloadDescriptor, ProtocolError> {
    command_payload_descriptor(SET_TOWN_FOCUS_COMMAND_ID).ok_or(
        ProtocolError::UnsupportedCommandPayload(SET_TOWN_FOCUS_COMMAND_ID),
    )
}

fn validate_entry_count(count: usize, maximum: usize) -> Result<(), ProtocolError> {
    if count > maximum || u16::try_from(count).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context: "TownFocusCommandPayload.allocation",
            actual: count,
            maximum,
        });
    }
    Ok(())
}

fn validate_component_length(length: usize, maximum: usize) -> Result<(), ProtocolError> {
    if length > maximum || u32::try_from(length).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context: "TownFocusCommandPayload.component",
            actual: length,
            maximum,
        });
    }
    Ok(())
}

fn read_component(
    bytes: &[u8],
    cursor: &mut usize,
    maximum: usize,
) -> Result<String, ProtocolError> {
    let length = read_u32(bytes, cursor, "TownFocusCommandPayload.component")? as usize;
    validate_component_length(length, maximum)?;
    let value = take(bytes, cursor, length, "TownFocusCommandPayload.component")?;
    std::str::from_utf8(value)
        .map(str::to_owned)
        .map_err(|_| ProtocolError::InvalidUtf8 {
            context: "TownFocusCommandPayload.component",
        })
}

fn read_u16(bytes: &[u8], cursor: &mut usize, context: &'static str) -> Result<u16, ProtocolError> {
    Ok(u16::from_le_bytes(
        take(bytes, cursor, ENTRY_COUNT_BYTES, context)?
            .try_into()
            .expect("fixed town-focus u16 slice"),
    ))
}

fn read_u32(bytes: &[u8], cursor: &mut usize, context: &'static str) -> Result<u32, ProtocolError> {
    Ok(u32::from_le_bytes(
        take(bytes, cursor, LENGTH_PREFIX_BYTES, context)?
            .try_into()
            .expect("fixed town-focus u32 slice"),
    ))
}

fn read_i32(bytes: &[u8], cursor: &mut usize, context: &'static str) -> Result<i32, ProtocolError> {
    Ok(i32::from_le_bytes(
        take(bytes, cursor, POINTS_BYTES, context)?
            .try_into()
            .expect("fixed town-focus i32 slice"),
    ))
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
#[path = "tests/town_focus_payload.rs"]
mod tests;
