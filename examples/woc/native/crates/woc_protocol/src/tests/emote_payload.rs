use super::*;

#[test]
fn source_order_is_stable_and_closed() {
    let ids = [
        EmoteId::Wave,
        EmoteId::Laugh,
        EmoteId::Question,
        EmoteId::Cheer,
        EmoteId::Dance,
        EmoteId::Point,
        EmoteId::Flex,
        EmoteId::Salute,
        EmoteId::Cry,
        EmoteId::Bow,
        EmoteId::Clap,
        EmoteId::Roar,
        EmoteId::Kneel,
    ];
    for (index, emote) in ids.into_iter().enumerate() {
        let payload = EmoteCommandPayload { emote };
        assert_eq!(payload.encode(), [(index + 1) as u8]);
        assert_eq!(EmoteCommandPayload::decode(&payload.encode()), Ok(payload));
    }
    assert_eq!(
        EmoteCommandPayload::decode(&[0]),
        Err(ProtocolError::InvalidEmoteId(0))
    );
    assert_eq!(
        EmoteCommandPayload::decode(&[14]),
        Err(ProtocolError::InvalidEmoteId(14))
    );
}
