use crate::{
    require_finite, validate_command_payload, ProtocolError, ARENA_AUGMENT_COMMAND_ID,
    ARENA_QUEUE_COMMAND_ID, DUEL_REQUEST_COMMAND_ID,
};

const NUMBER_BYTES: usize = 8;
const LENGTH_PREFIX_BYTES: usize = 4;
const ARENA_QUEUE_BYTES: usize = 1;
const ARENA_AUGMENT_MAX_UTF16_CODE_UNITS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArenaFormat {
    OneVOne,
    TwoVTwo,
    Fiesta,
    YumiThree,
    YumiFive,
}

impl ArenaFormat {
    const fn wire_code(self) -> u8 {
        match self {
            Self::OneVOne => 0,
            Self::TwoVTwo => 1,
            Self::Fiesta => 2,
            Self::YumiThree => 3,
            Self::YumiFive => 4,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::OneVOne),
            1 => Ok(Self::TwoVTwo),
            2 => Ok(Self::Fiesta),
            3 => Ok(Self::YumiThree),
            4 => Ok(Self::YumiFive),
            invalid => Err(ProtocolError::InvalidArenaFormat(invalid)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DuelRequestCommandPayload {
    pub target_id: f64,
}

impl DuelRequestCommandPayload {
    pub fn encode(self) -> Result<[u8; NUMBER_BYTES], ProtocolError> {
        let target_id =
            canonical_finite_f64("DuelRequestCommandPayload.target_id", self.target_id)?;
        let bytes = target_id.to_le_bytes();
        validate_command_payload(DUEL_REQUEST_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DUEL_REQUEST_COMMAND_ID, bytes)?;
        Ok(Self {
            target_id: read_finite_f64(bytes, 0, "DuelRequestCommandPayload.target_id")?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArenaQueueCommandPayload {
    pub format: ArenaFormat,
}

impl ArenaQueueCommandPayload {
    pub fn encode(self) -> Result<[u8; ARENA_QUEUE_BYTES], ProtocolError> {
        let bytes = [self.format.wire_code()];
        validate_command_payload(ARENA_QUEUE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(ARENA_QUEUE_COMMAND_ID, bytes)?;
        Ok(Self {
            format: ArenaFormat::from_wire_code(bytes[0])?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArenaAugmentCommandPayload {
    pub augment_id: String,
}

impl ArenaAugmentCommandPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        validate_arena_augment_id(&self.augment_id)?;
        let mut bytes = Vec::with_capacity(LENGTH_PREFIX_BYTES + self.augment_id.len());
        bytes.extend_from_slice(&(self.augment_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(self.augment_id.as_bytes());
        validate_command_payload(ARENA_AUGMENT_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(ARENA_AUGMENT_COMMAND_ID, bytes)?;
        Ok(Self {
            augment_id: decode_arena_augment_id(bytes)?.to_owned(),
        })
    }
}

pub(crate) fn validate_duel_request_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, 0, "DuelRequestCommandPayload.target_id")?;
    Ok(())
}

pub(crate) fn validate_arena_queue_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = ArenaFormat::from_wire_code(bytes[0])?;
    Ok(())
}

pub(crate) fn validate_arena_augment_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_arena_augment_id(bytes)?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(
    bytes: &[u8],
    offset: usize,
    context: &'static str,
) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[offset..offset + NUMBER_BYTES]
            .try_into()
            .expect("validated duel-arena payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

fn decode_arena_augment_id(bytes: &[u8]) -> Result<&str, ProtocolError> {
    if bytes.len() < LENGTH_PREFIX_BYTES {
        return Err(ProtocolError::TruncatedPayload {
            context: "ArenaAugmentCommandPayload.augment_id",
            needed: LENGTH_PREFIX_BYTES,
            remaining: bytes.len(),
        });
    }
    let declared = u32::from_le_bytes(
        bytes[..LENGTH_PREFIX_BYTES]
            .try_into()
            .expect("length-prefix slice has four bytes"),
    ) as usize;
    let actual = bytes.len() - LENGTH_PREFIX_BYTES;
    if declared != actual {
        return Err(ProtocolError::LengthMismatch { declared, actual });
    }
    let augment_id = std::str::from_utf8(&bytes[LENGTH_PREFIX_BYTES..]).map_err(|_| {
        ProtocolError::InvalidUtf8 {
            context: "ArenaAugmentCommandPayload.augment_id",
        }
    })?;
    validate_arena_augment_id(augment_id)?;
    Ok(augment_id)
}

fn validate_arena_augment_id(augment_id: &str) -> Result<(), ProtocolError> {
    // The source server enforces JavaScript UTF-16 units, not UTF-8 bytes.
    let utf16_code_units = augment_id.encode_utf16().count();
    if utf16_code_units > ARENA_AUGMENT_MAX_UTF16_CODE_UNITS {
        return Err(ProtocolError::CollectionTooLarge {
            context: "ArenaAugmentCommandPayload.augment_id_utf16_code_units",
            actual: utf16_code_units,
            maximum: ARENA_AUGMENT_MAX_UTF16_CODE_UNITS,
        });
    }
    Ok(())
}
