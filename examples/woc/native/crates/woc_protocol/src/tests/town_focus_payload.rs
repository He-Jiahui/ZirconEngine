use super::*;

#[test]
fn rejects_duplicate_wire_keys_and_oversized_components() {
    let mut duplicate = 2u16.to_le_bytes().to_vec();
    for points in [1i32, 2] {
        duplicate.extend_from_slice(&4u32.to_le_bytes());
        duplicate.extend_from_slice(b"hide");
        duplicate.extend_from_slice(&points.to_le_bytes());
    }
    assert!(matches!(
        TownFocusCommandPayload::decode(&duplicate),
        Err(ProtocolError::DuplicateCommandObjectKey { .. })
    ));

    let payload = TownFocusCommandPayload {
        allocation: vec![TownFocusAllocationEntry {
            component: "x".repeat(257),
            points: 1,
        }],
    };
    assert!(matches!(
        payload.encode(),
        Err(ProtocolError::CollectionTooLarge { .. })
    ));
}
