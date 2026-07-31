use crate::{
    require_finite, validate_command_payload, ProtocolError, BANK_DEPOSIT_COMMAND_ID,
    BANK_WITHDRAW_COMMAND_ID,
};

const SLOT_BYTES: usize = 8;
const COUNT_PRESENCE_BYTES: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BankAction {
    Deposit,
    Withdraw,
}

impl BankAction {
    const fn command_id(self) -> u16 {
        match self {
            Self::Deposit => BANK_DEPOSIT_COMMAND_ID,
            Self::Withdraw => BANK_WITHDRAW_COMMAND_ID,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BankSlotCommandPayload {
    pub slot: f64,
    pub count: Option<f64>,
}

impl BankSlotCommandPayload {
    pub fn encode(self, action: BankAction) -> Result<Vec<u8>, ProtocolError> {
        let slot = canonical_finite_f64("BankSlotCommandPayload.slot", self.slot)?;
        let count = self
            .count
            .map(|count| canonical_finite_f64("BankSlotCommandPayload.count", count))
            .transpose()?;
        let mut bytes = Vec::with_capacity(
            SLOT_BYTES + COUNT_PRESENCE_BYTES + usize::from(count.is_some()) * SLOT_BYTES,
        );
        bytes.extend_from_slice(&slot.to_le_bytes());
        match count {
            None => bytes.push(0),
            Some(count) => {
                bytes.push(1);
                bytes.extend_from_slice(&count.to_le_bytes());
            }
        }
        validate_command_payload(action.command_id(), &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8], action: BankAction) -> Result<Self, ProtocolError> {
        validate_command_payload(action.command_id(), bytes)?;
        let (slot, count) = read_payload(bytes)?;
        Ok(Self { slot, count })
    }
}

pub(crate) fn validate_bank_slot_optional_count_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_payload(bytes)?;
    Ok(())
}

fn read_payload(bytes: &[u8]) -> Result<(f64, Option<f64>), ProtocolError> {
    let slot = read_finite_f64(bytes, 0, "BankSlotCommandPayload.slot")?;
    let count = match bytes[SLOT_BYTES] {
        0 => None,
        1 => Some(read_finite_f64(
            bytes,
            SLOT_BYTES + COUNT_PRESENCE_BYTES,
            "BankSlotCommandPayload.count",
        )?),
        invalid => return Err(ProtocolError::InvalidBoolean(invalid)),
    };
    Ok((slot, count))
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
        bytes[offset..offset + SLOT_BYTES]
            .try_into()
            .expect("validated bank payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}
