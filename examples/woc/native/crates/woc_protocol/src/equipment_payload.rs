use crate::{
    command_payload_descriptor, validate_command_payload, ProtocolError, EQUIP_ITEM_COMMAND_ID,
    UNEQUIP_ITEM_COMMAND_ID,
};

const LENGTH_PREFIX_BYTES: usize = 4;
const SLOT_BYTES: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentSlot {
    Mainhand,
    Offhand,
    Helmet,
    Neck,
    Shoulder,
    Chest,
    Waist,
    Legs,
    Gloves,
    Feet,
    Ring1,
    Ring2,
}

impl EquipmentSlot {
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::Mainhand => 1,
            Self::Offhand => 2,
            Self::Helmet => 3,
            Self::Neck => 4,
            Self::Shoulder => 5,
            Self::Chest => 6,
            Self::Waist => 7,
            Self::Legs => 8,
            Self::Gloves => 9,
            Self::Feet => 10,
            Self::Ring1 => 11,
            Self::Ring2 => 12,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            1 => Ok(Self::Mainhand),
            2 => Ok(Self::Offhand),
            3 => Ok(Self::Helmet),
            4 => Ok(Self::Neck),
            5 => Ok(Self::Shoulder),
            6 => Ok(Self::Chest),
            7 => Ok(Self::Waist),
            8 => Ok(Self::Legs),
            9 => Ok(Self::Gloves),
            10 => Ok(Self::Feet),
            11 => Ok(Self::Ring1),
            12 => Ok(Self::Ring2),
            _ => Err(ProtocolError::InvalidEquipmentSlot(code)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquipItemPayload {
    pub item_id: String,
    pub slot: Option<EquipmentSlot>,
}

impl EquipItemPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = equip_descriptor()?;
        let item_bytes = self.item_id.as_bytes();
        validate_item_length(item_bytes.len(), descriptor.max_utf8_bytes)?;
        let mut bytes = Vec::with_capacity(LENGTH_PREFIX_BYTES + item_bytes.len() + SLOT_BYTES);
        bytes.extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(item_bytes);
        bytes.push(self.slot.map_or(0, EquipmentSlot::wire_code));
        validate_command_payload(EQUIP_ITEM_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(EQUIP_ITEM_COMMAND_ID, bytes)?;
        decode_equip_payload(bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnequipItemPayload {
    pub slot: EquipmentSlot,
}

impl UnequipItemPayload {
    pub fn encode(self) -> Result<[u8; 1], ProtocolError> {
        let bytes = [self.slot.wire_code()];
        validate_command_payload(UNEQUIP_ITEM_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(UNEQUIP_ITEM_COMMAND_ID, bytes)?;
        Ok(Self {
            slot: EquipmentSlot::from_wire_code(bytes[0])?,
        })
    }
}

pub(crate) fn validate_equip_item_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_equip_payload(bytes)?;
    Ok(())
}

pub(crate) fn validate_unequip_item_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = EquipmentSlot::from_wire_code(bytes[0])?;
    Ok(())
}

fn decode_equip_payload(bytes: &[u8]) -> Result<EquipItemPayload, ProtocolError> {
    let descriptor = equip_descriptor()?;
    let length_bytes = bytes
        .get(..LENGTH_PREFIX_BYTES)
        .ok_or(ProtocolError::TruncatedPayload {
            context: "EquipItemPayload.item_id length",
            needed: LENGTH_PREFIX_BYTES,
            remaining: bytes.len(),
        })?;
    let item_length = u32::from_le_bytes(
        length_bytes
            .try_into()
            .expect("equipment item length prefix has four bytes"),
    ) as usize;
    validate_item_length(item_length, descriptor.max_utf8_bytes)?;
    let item_end = LENGTH_PREFIX_BYTES + item_length;
    let expected = item_end + SLOT_BYTES;
    if bytes.len() != expected {
        return Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: EQUIP_ITEM_COMMAND_ID,
            actual: bytes.len(),
            expected,
        });
    }
    let item_id = std::str::from_utf8(&bytes[LENGTH_PREFIX_BYTES..item_end])
        .map_err(|_| ProtocolError::InvalidUtf8 {
            context: "EquipItemPayload.item_id",
        })?
        .to_owned();
    let slot = match bytes[item_end] {
        0 => None,
        code => Some(EquipmentSlot::from_wire_code(code)?),
    };
    Ok(EquipItemPayload { item_id, slot })
}

fn equip_descriptor() -> Result<&'static crate::CommandPayloadDescriptor, ProtocolError> {
    command_payload_descriptor(EQUIP_ITEM_COMMAND_ID).ok_or(
        ProtocolError::UnsupportedCommandPayload(EQUIP_ITEM_COMMAND_ID),
    )
}

fn validate_item_length(length: usize, maximum: usize) -> Result<(), ProtocolError> {
    if length > maximum || u32::try_from(length).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context: "EquipItemPayload.item_id",
            actual: length,
            maximum,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_automatic_and_aimed_equips() {
        for slot in [
            None,
            Some(EquipmentSlot::Helmet),
            Some(EquipmentSlot::Ring2),
        ] {
            let payload = EquipItemPayload {
                item_id: "cryptbone_helm".to_owned(),
                slot,
            };
            let encoded = payload.clone().encode().expect("payload encodes");
            assert_eq!(EquipItemPayload::decode(&encoded), Ok(payload));
        }
    }

    #[test]
    fn unequip_requires_a_live_equipment_slot() {
        assert!(matches!(
            UnequipItemPayload::decode(&[0]),
            Err(ProtocolError::InvalidEquipmentSlot(0))
        ));
        assert_eq!(
            UnequipItemPayload::decode(&[EquipmentSlot::Feet.wire_code()]),
            Ok(UnequipItemPayload {
                slot: EquipmentSlot::Feet,
            })
        );
    }
}
