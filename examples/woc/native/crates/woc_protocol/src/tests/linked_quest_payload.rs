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
