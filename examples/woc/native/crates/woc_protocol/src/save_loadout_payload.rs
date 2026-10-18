use crate::{
    validate_command_payload, ApplyTalentsCommandPayload, ProtocolError, SAVE_LOADOUT_COMMAND_ID,
};

pub const MAX_LOADOUT_NAME_UTF16_CODE_UNITS: usize = 24;
pub const MAX_LOADOUT_NAME_UTF8_BYTES: usize = 96;
pub const MAX_LOADOUT_ACTION_BAR_SLOTS: usize = 22;
pub const MAX_LOADOUT_ABILITY_ID_UTF8_BYTES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveLoadoutCommandPayload {
    pub name: String,
    pub allocation: Option<ApplyTalentsCommandPayload>,
    pub action_bar: Vec<Option<String>>,
}

impl SaveLoadoutCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        validate_name(&self.name)?;
        validate_action_bar(&self.action_bar)?;

        let mut bytes = Vec::with_capacity(6 + self.name.len() + self.action_bar.len() * 5);
        push_utf8(&mut bytes, &self.name);
        match self.allocation {
            Some(allocation) => {
                bytes.push(1);
                bytes.extend_from_slice(&allocation.encode()?);
            }
            None => bytes.push(0),
        }
        bytes.push(self.action_bar.len() as u8);
        for ability_id in &self.action_bar {
            match ability_id {
                Some(ability_id) => {
                    bytes.push(1);
                    push_utf8(&mut bytes, ability_id);
                }
                None => bytes.push(0),
            }
        }
        validate_command_payload(SAVE_LOADOUT_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(SAVE_LOADOUT_COMMAND_ID, bytes)?;
        parse_payload(bytes)
    }
}

pub(crate) fn validate_save_loadout_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    parse_payload(bytes).map(|_| ())
}

fn parse_payload(bytes: &[u8]) -> Result<SaveLoadoutCommandPayload, ProtocolError> {
    let (name, mut cursor) = read_utf8(
        bytes,
        0,
        MAX_LOADOUT_NAME_UTF8_BYTES,
        "SaveLoadoutCommandPayload.name",
    )?;
    validate_name(name)?;

    let allocation_present = *take(
        bytes,
        cursor,
        1,
        "SaveLoadoutCommandPayload.allocation_presence",
    )?
    .first()
    .expect("one-byte presence slice");
    cursor += 1;
    let allocation = match allocation_present {
        0 => None,
        1 => {
            let encoded = take(bytes, cursor, 14, "SaveLoadoutCommandPayload.allocation")?;
            cursor += 14;
            Some(ApplyTalentsCommandPayload::decode(encoded)?)
        }
        value => return Err(ProtocolError::InvalidBoolean(value)),
    };

    let slot_count = usize::from(
        *take(
            bytes,
            cursor,
            1,
            "SaveLoadoutCommandPayload.action_bar_count",
        )?
        .first()
        .expect("one-byte count slice"),
    );
    cursor += 1;
    if slot_count > MAX_LOADOUT_ACTION_BAR_SLOTS {
        return Err(ProtocolError::CollectionTooLarge {
            context: "SaveLoadoutCommandPayload.action_bar",
            actual: slot_count,
            maximum: MAX_LOADOUT_ACTION_BAR_SLOTS,
        });
    }

    let mut action_bar = Vec::with_capacity(slot_count);
    for _ in 0..slot_count {
        let present = *take(
            bytes,
            cursor,
            1,
            "SaveLoadoutCommandPayload.action_bar_presence",
        )?
        .first()
        .expect("one-byte presence slice");
        cursor += 1;
        match present {
            0 => action_bar.push(None),
            1 => {
                let (ability_id, consumed) = read_utf8(
                    bytes,
                    cursor,
                    MAX_LOADOUT_ABILITY_ID_UTF8_BYTES,
                    "SaveLoadoutCommandPayload.action_bar_ability_id",
                )?;
                cursor = consumed;
                action_bar.push(Some(ability_id.to_owned()));
            }
            value => return Err(ProtocolError::InvalidBoolean(value)),
        }
    }
    if cursor != bytes.len() {
        return Err(ProtocolError::TrailingPayload {
            remaining: bytes.len() - cursor,
        });
    }

    Ok(SaveLoadoutCommandPayload {
        name: name.to_owned(),
        allocation,
        action_bar,
    })
}

fn validate_name(name: &str) -> Result<(), ProtocolError> {
    let utf16_code_units = name.encode_utf16().count();
    if utf16_code_units > MAX_LOADOUT_NAME_UTF16_CODE_UNITS {
        return Err(ProtocolError::CollectionTooLarge {
            context: "SaveLoadoutCommandPayload.name_utf16_code_units",
            actual: utf16_code_units,
            maximum: MAX_LOADOUT_NAME_UTF16_CODE_UNITS,
        });
    }
    if name.len() > MAX_LOADOUT_NAME_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context: "SaveLoadoutCommandPayload.name_utf8_bytes",
            actual: name.len(),
            maximum: MAX_LOADOUT_NAME_UTF8_BYTES,
        });
    }
    Ok(())
}

fn validate_action_bar(action_bar: &[Option<String>]) -> Result<(), ProtocolError> {
    if action_bar.len() > MAX_LOADOUT_ACTION_BAR_SLOTS {
        return Err(ProtocolError::CollectionTooLarge {
            context: "SaveLoadoutCommandPayload.action_bar",
            actual: action_bar.len(),
            maximum: MAX_LOADOUT_ACTION_BAR_SLOTS,
        });
    }
    for ability_id in action_bar.iter().flatten() {
        if ability_id.len() > MAX_LOADOUT_ABILITY_ID_UTF8_BYTES {
            return Err(ProtocolError::CollectionTooLarge {
                context: "SaveLoadoutCommandPayload.action_bar_ability_id",
                actual: ability_id.len(),
                maximum: MAX_LOADOUT_ABILITY_ID_UTF8_BYTES,
            });
        }
    }
    Ok(())
}

fn push_utf8(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn read_utf8<'a>(
    bytes: &'a [u8],
    cursor: usize,
    maximum: usize,
    context: &'static str,
) -> Result<(&'a str, usize), ProtocolError> {
    let length_bytes = take(bytes, cursor, 4, context)?;
    let length = u32::from_le_bytes(
        length_bytes
            .try_into()
            .expect("four-byte UTF-8 length prefix"),
    ) as usize;
    if length > maximum {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: length,
            maximum,
        });
    }
    let start = cursor + 4;
    let encoded = take(bytes, start, length, context)?;
    let value = std::str::from_utf8(encoded).map_err(|_| ProtocolError::InvalidUtf8 { context })?;
    Ok((value, start + length))
}

fn take<'a>(
    bytes: &'a [u8],
    offset: usize,
    length: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let remaining = bytes.len().saturating_sub(offset);
    if offset > bytes.len() || length > remaining {
        return Err(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining,
        });
    }
    Ok(&bytes[offset..offset + length])
}
