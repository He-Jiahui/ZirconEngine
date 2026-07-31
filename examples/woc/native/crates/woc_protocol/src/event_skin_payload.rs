use crate::{require_finite, validate_command_payload, ProtocolError, CLAIM_EVENT_SKIN_COMMAND_ID};

const EVENT_SKIN_PAYLOAD_BYTES: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct EventSkinPayload {
    pub skin: f64,
}

impl EventSkinPayload {
    pub fn encode(self) -> Result<[u8; EVENT_SKIN_PAYLOAD_BYTES], ProtocolError> {
        let skin = canonical_finite_f64("EventSkinPayload.skin", self.skin)?;
        let bytes = skin.to_le_bytes();
        validate_command_payload(CLAIM_EVENT_SKIN_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CLAIM_EVENT_SKIN_COMMAND_ID, bytes)?;
        Ok(Self {
            skin: read_finite_f64(bytes, "EventSkinPayload.skin")?,
        })
    }
}

pub(crate) fn validate_event_skin_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, "EventSkinPayload.skin")?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(bytes: &[u8], context: &'static str) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[..EVENT_SKIN_PAYLOAD_BYTES]
            .try_into()
            .expect("validated event-skin payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}
