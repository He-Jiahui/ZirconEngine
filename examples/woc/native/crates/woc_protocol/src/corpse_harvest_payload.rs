use crate::{
    command_payload_descriptor, corpse_harvest_contract::is_valid_corpse_harvest_component_code,
    validate_command_payload, ProtocolError, HARVEST_CORPSE_COMMAND_ID,
};

const TARGET_ID_BYTES: usize = 8;
const COMPONENT_COUNT_BYTES: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarvestCorpseCommandPayload {
    pub target_id: u64,
    // Zero is a deliberate lossless projection of a source component string
    // that does not match the generated current-head tag catalog.
    pub component_codes: Vec<u8>,
}

impl HarvestCorpseCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        validate_component_codes(&self.component_codes)?;
        let mut bytes = Vec::with_capacity(
            TARGET_ID_BYTES + COMPONENT_COUNT_BYTES + self.component_codes.len(),
        );
        bytes.extend_from_slice(&self.target_id.to_le_bytes());
        bytes.push(self.component_codes.len() as u8);
        bytes.extend_from_slice(&self.component_codes);
        validate_command_payload(HARVEST_CORPSE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(HARVEST_CORPSE_COMMAND_ID, bytes)?;
        let target_id = u64::from_le_bytes(
            bytes[..TARGET_ID_BYTES]
                .try_into()
                .expect("validated corpse-harvest payload has a target id"),
        );
        let count = bytes[TARGET_ID_BYTES] as usize;
        Ok(Self {
            target_id,
            component_codes: bytes[TARGET_ID_BYTES + COMPONENT_COUNT_BYTES
                ..TARGET_ID_BYTES + COMPONENT_COUNT_BYTES + count]
                .to_vec(),
        })
    }
}

pub(crate) fn validate_corpse_harvest_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let count = bytes[TARGET_ID_BYTES] as usize;
    let maximum = command_payload_descriptor(HARVEST_CORPSE_COMMAND_ID)
        .ok_or(ProtocolError::UnsupportedCommandPayload(
            HARVEST_CORPSE_COMMAND_ID,
        ))?
        .max_collection_entries;
    if count > maximum {
        return Err(ProtocolError::InvalidCorpseHarvestComponentCount(
            bytes[TARGET_ID_BYTES],
        ));
    }
    let expected = TARGET_ID_BYTES + COMPONENT_COUNT_BYTES + count;
    if bytes.len() < expected {
        return Err(ProtocolError::TruncatedPayload {
            context: "HarvestCorpseCommandPayload.component_codes",
            needed: count,
            remaining: bytes
                .len()
                .saturating_sub(TARGET_ID_BYTES + COMPONENT_COUNT_BYTES),
        });
    }
    if bytes.len() > expected {
        return Err(ProtocolError::TrailingPayload {
            remaining: bytes.len() - expected,
        });
    }
    validate_component_codes(&bytes[TARGET_ID_BYTES + COMPONENT_COUNT_BYTES..expected])
}

fn validate_component_codes(component_codes: &[u8]) -> Result<(), ProtocolError> {
    let maximum = command_payload_descriptor(HARVEST_CORPSE_COMMAND_ID)
        .ok_or(ProtocolError::UnsupportedCommandPayload(
            HARVEST_CORPSE_COMMAND_ID,
        ))?
        .max_collection_entries;
    if component_codes.len() > maximum {
        return Err(ProtocolError::CollectionTooLarge {
            context: "HarvestCorpseCommandPayload.component_codes",
            actual: component_codes.len(),
            maximum,
        });
    }
    for &code in component_codes {
        if !is_valid_corpse_harvest_component_code(code) {
            return Err(ProtocolError::InvalidCorpseHarvestComponentCode(code));
        }
    }
    Ok(())
}
