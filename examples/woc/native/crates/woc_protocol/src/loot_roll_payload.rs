use crate::{require_finite, validate_command_payload, ProtocolError, LOOT_ROLL_COMMAND_ID};

const LOOT_ROLL_ID_BYTES: usize = 8;
const LOOT_ROLL_PAYLOAD_BYTES: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LootRollChoice {
    Need,
    Greed,
    Pass,
}

impl LootRollChoice {
    const fn code(self) -> u8 {
        match self {
            Self::Need => 0,
            Self::Greed => 1,
            Self::Pass => 2,
        }
    }

    fn decode(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Need),
            1 => Ok(Self::Greed),
            2 => Ok(Self::Pass),
            other => Err(ProtocolError::InvalidLootRollChoice(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LootRollPayload {
    pub roll_id: f64,
    pub choice: LootRollChoice,
}

impl LootRollPayload {
    pub fn encode(self) -> Result<[u8; LOOT_ROLL_PAYLOAD_BYTES], ProtocolError> {
        let roll_id = canonical_finite_f64("LootRollPayload.roll_id", self.roll_id)?;
        let mut bytes = [0; LOOT_ROLL_PAYLOAD_BYTES];
        bytes[..LOOT_ROLL_ID_BYTES].copy_from_slice(&roll_id.to_le_bytes());
        bytes[LOOT_ROLL_ID_BYTES] = self.choice.code();
        validate_command_payload(LOOT_ROLL_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(LOOT_ROLL_COMMAND_ID, bytes)?;
        Ok(Self {
            roll_id: read_finite_f64(bytes, "LootRollPayload.roll_id")?,
            choice: LootRollChoice::decode(bytes[LOOT_ROLL_ID_BYTES])?,
        })
    }
}

pub(crate) fn validate_loot_roll_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "LootRollPayload.roll_id")?;
    let _ = LootRollChoice::decode(bytes[LOOT_ROLL_ID_BYTES])?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..LOOT_ROLL_ID_BYTES]
            .try_into()
            .expect("validated loot-roll payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}
