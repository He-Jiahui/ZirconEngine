use crate::{require_finite, validate_command_payload, ProtocolError, MASTER_ASSIGN_COMMAND_ID};

const F64_BYTES: usize = 8;
const COUNT_BYTES: usize = 1;
pub const MAX_TARGET_PIDS: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct MasterLootAssignmentPayload {
    pub roll_id: f64,
    pub target_pids: Vec<f64>,
}

impl MasterLootAssignmentPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        validate_target_count(self.target_pids.len())?;
        let roll_id = canonical_finite_f64("MasterLootAssignmentPayload.roll_id", self.roll_id)?;
        let mut bytes =
            Vec::with_capacity(F64_BYTES + COUNT_BYTES + self.target_pids.len() * F64_BYTES);
        bytes.extend_from_slice(&roll_id.to_le_bytes());
        bytes.push(self.target_pids.len() as u8);
        for target_pid in self.target_pids {
            let target_pid =
                canonical_finite_f64("MasterLootAssignmentPayload.target_pid", target_pid)?;
            bytes.extend_from_slice(&target_pid.to_le_bytes());
        }
        validate_command_payload(MASTER_ASSIGN_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(MASTER_ASSIGN_COMMAND_ID, bytes)?;
        decode_payload(bytes)
    }
}

pub(crate) fn validate_master_loot_assignment_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = decode_payload(bytes)?;
    Ok(())
}

fn decode_payload(bytes: &[u8]) -> Result<MasterLootAssignmentPayload, ProtocolError> {
    let roll_id = read_finite_f64(bytes, 0, "MasterLootAssignmentPayload.roll_id")?;
    let target_count = bytes[F64_BYTES] as usize;
    validate_target_count(target_count)?;
    let expected = F64_BYTES + COUNT_BYTES + target_count * F64_BYTES;
    if bytes.len() != expected {
        return Err(ProtocolError::InvalidCommandPayloadLength {
            command_id: MASTER_ASSIGN_COMMAND_ID,
            actual: bytes.len(),
            expected,
        });
    }
    let mut target_pids = Vec::with_capacity(target_count);
    for index in 0..target_count {
        target_pids.push(read_finite_f64(
            bytes,
            F64_BYTES + COUNT_BYTES + index * F64_BYTES,
            "MasterLootAssignmentPayload.target_pid",
        )?);
    }
    Ok(MasterLootAssignmentPayload {
        roll_id,
        target_pids,
    })
}

fn validate_target_count(count: usize) -> Result<(), ProtocolError> {
    if count > MAX_TARGET_PIDS {
        return Err(ProtocolError::CollectionTooLarge {
            context: "MasterLootAssignmentPayload.target_pids",
            actual: count,
            maximum: MAX_TARGET_PIDS,
        });
    }
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
        bytes[offset..offset + F64_BYTES]
            .try_into()
            .expect("validated master-loot assignment payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_empty_single_and_raid_target_sets() {
        for target_pids in [
            vec![],
            vec![7.0],
            (1..=MAX_TARGET_PIDS).map(|value| value as f64).collect(),
        ] {
            let payload = MasterLootAssignmentPayload {
                roll_id: 42.0,
                target_pids,
            };
            let encoded = payload.clone().encode().expect("payload encodes");
            assert_eq!(MasterLootAssignmentPayload::decode(&encoded), Ok(payload));
        }
    }

    #[test]
    fn rejects_count_length_mismatch() {
        let mut bytes = Vec::from(42.0f64.to_le_bytes());
        bytes.push(1);
        assert!(matches!(
            MasterLootAssignmentPayload::decode(&bytes),
            Err(ProtocolError::InvalidCommandPayloadLength { .. })
        ));
    }
}
