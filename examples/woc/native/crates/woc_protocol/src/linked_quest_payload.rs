use crate::{
    command_payload_descriptor, require_finite, validate_command_payload, ProtocolError,
    LINKED_QUEST_ACCEPT_COMMAND_ID,
};

const LENGTH_PREFIX_BYTES: usize = 4;
const F64_BYTES: usize = 8;
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Clone, Debug, PartialEq)]
pub struct LinkedQuestAcceptancePayload {
    pub quest_id: String,
    pub sharer_pid: f64,
}

impl LinkedQuestAcceptancePayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        let descriptor = descriptor()?;
        let quest_bytes = self.quest_id.as_bytes();
        validate_quest_length(quest_bytes.len(), descriptor.max_utf8_bytes)?;
        let sharer_pid = validate_sharer_pid(self.sharer_pid)?;
        let mut bytes = Vec::with_capacity(LENGTH_PREFIX_BYTES + quest_bytes.len() + F64_BYTES);
        bytes.extend_from_slice(&(quest_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(quest_bytes);
        bytes.extend_from_slice(&sharer_pid.to_le_bytes());
        validate_command_payload(LINKED_QUEST_ACCEPT_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(LINKED_QUEST_ACCEPT_COMMAND_ID, bytes)?;
        decode_payload(bytes)
    }
}

pub(crate) fn validate_linked_quest_acceptance_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_payload(bytes)?;
    Ok(())
}

fn decode_payload(bytes: &[u8]) -> Result<LinkedQuestAcceptancePayload, ProtocolError> {
    let descriptor = descriptor()?;
    let length_bytes = bytes
        .get(..LENGTH_PREFIX_BYTES)
        .ok_or(ProtocolError::TruncatedPayload {
            context: "LinkedQuestAcceptancePayload.quest_id length",
            needed: LENGTH_PREFIX_BYTES,
            remaining: bytes.len(),
        })?;
    let quest_length = u32::from_le_bytes(
        length_bytes
            .try_into()
            .expect("linked-quest length prefix has four bytes"),
    ) as usize;
    validate_quest_length(quest_length, descriptor.max_utf8_bytes)?;
    let quest_end = LENGTH_PREFIX_BYTES + quest_length;
    let expected = quest_end + F64_BYTES;
    if bytes.len() != expected {
        return Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: LINKED_QUEST_ACCEPT_COMMAND_ID,
            actual: bytes.len(),
            expected,
        });
    }
    let quest_id = std::str::from_utf8(&bytes[LENGTH_PREFIX_BYTES..quest_end])
        .map_err(|_| ProtocolError::InvalidUtf8 {
            context: "LinkedQuestAcceptancePayload.quest_id",
        })?
        .to_owned();
    let sharer_pid = validate_sharer_pid(f64::from_le_bytes(
        bytes[quest_end..expected]
            .try_into()
            .expect("linked-quest sharer pid has eight bytes"),
    ))?;
    Ok(LinkedQuestAcceptancePayload {
        quest_id,
        sharer_pid,
    })
}

fn descriptor() -> Result<&'static crate::CommandPayloadDescriptor, ProtocolError> {
    command_payload_descriptor(LINKED_QUEST_ACCEPT_COMMAND_ID).ok_or(
        ProtocolError::UnsupportedCommandPayload(LINKED_QUEST_ACCEPT_COMMAND_ID),
    )
}

fn validate_quest_length(length: usize, maximum: usize) -> Result<(), ProtocolError> {
    if length > maximum || u32::try_from(length).is_err() {
        return Err(ProtocolError::CollectionTooLarge {
            context: "LinkedQuestAcceptancePayload.quest_id",
            actual: length,
            maximum,
        });
    }
    Ok(())
}

fn validate_sharer_pid(value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite("LinkedQuestAcceptancePayload.sharer_pid", value)?;
    if value <= 0.0 || value.fract() != 0.0 || value > MAX_SAFE_INTEGER {
        return Err(ProtocolError::InvalidEntityId {
            context: "LinkedQuestAcceptancePayload.sharer_pid",
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_source_quest_and_sharer_fields() {
        let payload = LinkedQuestAcceptancePayload {
            quest_id: "q_wolves".to_owned(),
            sharer_pid: 2.0,
        };
        let encoded = payload.clone().encode().expect("payload encodes");
        assert_eq!(LinkedQuestAcceptancePayload::decode(&encoded), Ok(payload));
    }

    #[test]
    fn rejects_non_integral_and_zero_sharer_ids() {
        for sharer_pid in [0.0, -1.0, 1.5] {
            assert!(matches!(
                LinkedQuestAcceptancePayload {
                    quest_id: "q_wolves".to_owned(),
                    sharer_pid,
                }
                .encode(),
                Err(ProtocolError::InvalidEntityId { .. })
            ));
        }
    }
}
