use crate::{require_finite, validate_command_payload, ProtocolError, TRADE_REQUEST_COMMAND_ID};

const NUMBER_BYTES: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct TradeRequestCommandPayload {
    pub target_id: f64,
}

impl TradeRequestCommandPayload {
    pub fn encode(self) -> Result<[u8; NUMBER_BYTES], ProtocolError> {
        let target_id =
            canonical_finite_f64("TradeRequestCommandPayload.target_id", self.target_id)?;
        let bytes = target_id.to_le_bytes();
        validate_command_payload(TRADE_REQUEST_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(TRADE_REQUEST_COMMAND_ID, bytes)?;
        Ok(Self {
            target_id: read_finite_f64(bytes, "TradeRequestCommandPayload.target_id")?,
        })
    }
}

pub(crate) fn validate_trade_request_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "TradeRequestCommandPayload.target_id")?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..NUMBER_BYTES]
            .try_into()
            .expect("validated trade payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}
