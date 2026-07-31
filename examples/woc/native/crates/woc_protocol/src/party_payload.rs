use crate::{
    require_finite, validate_command_payload, ProtocolError, PARTY_CLEAR_MARKER_COMMAND_ID,
    PARTY_READY_RESPOND_COMMAND_ID, PARTY_SET_LOOT_MASTER_COMMAND_ID, PARTY_SET_MARKER_COMMAND_ID,
};

const BOOLEAN_BYTES: usize = 1;
const NUMBER_BYTES: usize = 8;
const PARTY_LOOT_MASTER_BYTES: usize = BOOLEAN_BYTES + NUMBER_BYTES + 1;
const PARTY_MARKER_BYTES: usize = NUMBER_BYTES * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MasterLootThreshold {
    Uncommon,
    Rare,
    Epic,
}

impl MasterLootThreshold {
    const fn wire_code(self) -> u8 {
        match self {
            Self::Uncommon => 0,
            Self::Rare => 1,
            Self::Epic => 2,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Uncommon),
            1 => Ok(Self::Rare),
            2 => Ok(Self::Epic),
            invalid => Err(ProtocolError::InvalidMasterLootThreshold(invalid)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartyLootMasterCommandPayload {
    pub enabled: bool,
    pub looter: f64,
    pub threshold: MasterLootThreshold,
}

impl PartyLootMasterCommandPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        let looter = canonical_finite_f64("PartyLootMasterCommandPayload.looter", self.looter)?;
        let mut bytes = Vec::with_capacity(PARTY_LOOT_MASTER_BYTES);
        bytes.push(u8::from(self.enabled));
        bytes.extend_from_slice(&looter.to_le_bytes());
        bytes.push(self.threshold.wire_code());
        validate_command_payload(PARTY_SET_LOOT_MASTER_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PARTY_SET_LOOT_MASTER_COMMAND_ID, bytes)?;
        Ok(Self {
            enabled: decode_boolean(bytes[0])?,
            looter: read_finite_f64(bytes, BOOLEAN_BYTES, "PartyLootMasterCommandPayload.looter")?,
            threshold: MasterLootThreshold::from_wire_code(bytes[BOOLEAN_BYTES + NUMBER_BYTES])?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartyMarkerCommandPayload {
    pub entity_id: f64,
    pub marker_id: f64,
}

impl PartyMarkerCommandPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        let entity_id =
            canonical_finite_f64("PartyMarkerCommandPayload.entity_id", self.entity_id)?;
        let marker_id =
            canonical_finite_f64("PartyMarkerCommandPayload.marker_id", self.marker_id)?;
        let mut bytes = Vec::with_capacity(PARTY_MARKER_BYTES);
        bytes.extend_from_slice(&entity_id.to_le_bytes());
        bytes.extend_from_slice(&marker_id.to_le_bytes());
        validate_command_payload(PARTY_SET_MARKER_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PARTY_SET_MARKER_COMMAND_ID, bytes)?;
        Ok(Self {
            entity_id: read_finite_f64(bytes, 0, "PartyMarkerCommandPayload.entity_id")?,
            marker_id: read_finite_f64(bytes, NUMBER_BYTES, "PartyMarkerCommandPayload.marker_id")?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartyMarkerClearCommandPayload {
    pub entity_id: f64,
}

impl PartyMarkerClearCommandPayload {
    pub fn encode(self) -> Result<[u8; NUMBER_BYTES], ProtocolError> {
        let entity_id =
            canonical_finite_f64("PartyMarkerClearCommandPayload.entity_id", self.entity_id)?;
        let bytes = entity_id.to_le_bytes();
        validate_command_payload(PARTY_CLEAR_MARKER_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PARTY_CLEAR_MARKER_COMMAND_ID, bytes)?;
        Ok(Self {
            entity_id: read_finite_f64(bytes, 0, "PartyMarkerClearCommandPayload.entity_id")?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadyCheckRespondCommandPayload {
    pub ready: bool,
}

impl ReadyCheckRespondCommandPayload {
    pub const fn encode(self) -> [u8; BOOLEAN_BYTES] {
        [if self.ready { 1 } else { 0 }]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(PARTY_READY_RESPOND_COMMAND_ID, bytes)?;
        Ok(Self {
            ready: decode_boolean(bytes[0])?,
        })
    }
}

pub(crate) fn validate_party_loot_master_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_boolean(bytes[0])?;
    let _ = read_finite_f64(bytes, BOOLEAN_BYTES, "PartyLootMasterCommandPayload.looter")?;
    let _ = MasterLootThreshold::from_wire_code(bytes[BOOLEAN_BYTES + NUMBER_BYTES])?;
    Ok(())
}

pub(crate) fn validate_party_marker_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, 0, "PartyMarkerCommandPayload.entity_id")?;
    let _ = read_finite_f64(bytes, NUMBER_BYTES, "PartyMarkerCommandPayload.marker_id")?;
    Ok(())
}

pub(crate) fn validate_party_marker_clear_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, 0, "PartyMarkerClearCommandPayload.entity_id")?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn decode_boolean(value: u8) -> Result<bool, ProtocolError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}

fn read_finite_f64(
    bytes: &[u8],
    offset: usize,
    context: &'static str,
) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[offset..offset + NUMBER_BYTES]
            .try_into()
            .expect("validated party payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}
