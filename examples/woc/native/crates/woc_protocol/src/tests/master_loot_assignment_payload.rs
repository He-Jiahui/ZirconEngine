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
