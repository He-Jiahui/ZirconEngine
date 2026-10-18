use crate::{
    command_payload_descriptor, validate_command_payload, ProtocolError,
    CHANGE_WEAPON_SKIN_COMMAND_ID,
};

const APPLY_MODE: u8 = 1;
const DETACH_MODE: u8 = 0;
const LENGTH_PREFIX_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponSkinType {
    Sword,
    Axe,
    Mace,
    Dagger,
    Staff,
    Wand,
    Bow,
    Crossbow,
}

impl WeaponSkinType {
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::Sword => 1,
            Self::Axe => 2,
            Self::Mace => 3,
            Self::Dagger => 4,
            Self::Staff => 5,
            Self::Wand => 6,
            Self::Bow => 7,
            Self::Crossbow => 8,
        }
    }

    pub fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            1 => Ok(Self::Sword),
            2 => Ok(Self::Axe),
            3 => Ok(Self::Mace),
            4 => Ok(Self::Dagger),
            5 => Ok(Self::Staff),
            6 => Ok(Self::Wand),
            7 => Ok(Self::Bow),
            8 => Ok(Self::Crossbow),
            invalid => Err(ProtocolError::InvalidWeaponSkinType(invalid)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WeaponSkinChange {
    Apply { skin_id: String },
    Detach { weapon_type: WeaponSkinType },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeWeaponSkinCommandPayload {
    pub change: WeaponSkinChange,
}

impl ChangeWeaponSkinCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut bytes = Vec::new();
        match &self.change {
            WeaponSkinChange::Apply { skin_id } => {
                validate_skin_id_length(skin_id.len())?;
                bytes.push(APPLY_MODE);
                bytes.extend_from_slice(&(skin_id.len() as u32).to_le_bytes());
                bytes.extend_from_slice(skin_id.as_bytes());
            }
            WeaponSkinChange::Detach { weapon_type } => {
                bytes.push(DETACH_MODE);
                bytes.push(weapon_type.wire_code());
            }
        }
        validate_command_payload(CHANGE_WEAPON_SKIN_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CHANGE_WEAPON_SKIN_COMMAND_ID, bytes)?;
        let change = if bytes[0] == APPLY_MODE {
            let length = u32::from_le_bytes(
                bytes[1..5]
                    .try_into()
                    .expect("validated weapon-skin apply payload has a length prefix"),
            ) as usize;
            let skin_id = std::str::from_utf8(&bytes[5..5 + length])
                .expect("validated weapon-skin apply payload contains UTF-8")
                .to_owned();
            WeaponSkinChange::Apply { skin_id }
        } else {
            WeaponSkinChange::Detach {
                weapon_type: WeaponSkinType::from_wire_code(bytes[1])?,
            }
        };
        Ok(Self { change })
    }
}

pub(crate) fn validate_weapon_skin_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    match bytes[0] {
        DETACH_MODE => {
            if bytes.len() != 2 {
                return Err(ProtocolError::TrailingPayload {
                    remaining: bytes.len().saturating_sub(2),
                });
            }
            let _ = WeaponSkinType::from_wire_code(bytes[1])?;
            Ok(())
        }
        APPLY_MODE => validate_apply_payload(bytes),
        invalid => Err(ProtocolError::InvalidWeaponSkinMode(invalid)),
    }
}

fn validate_apply_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    if bytes.len() < 1 + LENGTH_PREFIX_BYTES {
        return Err(ProtocolError::TruncatedPayload {
            context: "ChangeWeaponSkinCommandPayload.skin_id",
            needed: 1 + LENGTH_PREFIX_BYTES,
            remaining: bytes.len(),
        });
    }
    let length = u32::from_le_bytes(
        bytes[1..5]
            .try_into()
            .expect("weapon-skin apply payload length prefix is present"),
    ) as usize;
    validate_skin_id_length(length)?;
    let expected = 1 + LENGTH_PREFIX_BYTES + length;
    if bytes.len() < expected {
        return Err(ProtocolError::TruncatedPayload {
            context: "ChangeWeaponSkinCommandPayload.skin_id",
            needed: length,
            remaining: bytes.len() - 1 - LENGTH_PREFIX_BYTES,
        });
    }
    if bytes.len() > expected {
        return Err(ProtocolError::TrailingPayload {
            remaining: bytes.len() - expected,
        });
    }
    std::str::from_utf8(&bytes[5..expected])
        .map(|_| ())
        .map_err(|_| ProtocolError::InvalidUtf8 {
            context: "ChangeWeaponSkinCommandPayload.skin_id",
        })
}

fn validate_skin_id_length(length: usize) -> Result<(), ProtocolError> {
    let maximum = command_payload_descriptor(CHANGE_WEAPON_SKIN_COMMAND_ID)
        .ok_or(ProtocolError::UnsupportedCommandPayload(
            CHANGE_WEAPON_SKIN_COMMAND_ID,
        ))?
        .max_utf8_bytes;
    if length > maximum || u32::try_from(length).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context: "ChangeWeaponSkinCommandPayload.skin_id",
            actual: length,
            maximum,
        });
    }
    Ok(())
}
