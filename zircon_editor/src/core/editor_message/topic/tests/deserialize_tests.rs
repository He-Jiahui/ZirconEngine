use super::EditorTopic;
use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessageDelivery, EditorMessagePayload,
    EditorMessageProtocol,
};

#[test]
fn deserialized_topic_obeys_the_public_topic_admission_rules() {
    for invalid in [
        "",
        "editor",
        ".document",
        "editor.",
        "editor..document",
        "Editor.document",
        "editor.Document",
        "editor.例",
    ] {
        let wire = serde_json::to_string(invalid).expect("topic string should serialize");
        assert!(
            serde_json::from_str::<EditorTopic>(&wire).is_err(),
            "deserialization accepted invalid topic {invalid:?}"
        );
    }

    let topic = EditorTopic::parse("editor.document").expect("valid topic");
    let wire = serde_json::to_string(&topic).expect("topic should serialize as a string");
    assert_eq!(wire, r#""editor.document""#);
    assert_eq!(serde_json::from_str::<EditorTopic>(&wire).unwrap(), topic);
}

#[test]
fn full_delivery_rejects_invalid_embedded_topic_without_changing_valid_wire_shape() {
    let delivery = EditorMessageDelivery::new(
        EditorMessageProtocol::Publish,
        EditorTopic::parse("editor.document").unwrap(),
        EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Opened {
            doc: DocumentId::new(7),
        })),
    );
    let mut wire = serde_json::to_value(&delivery).expect("delivery should serialize");
    assert_eq!(wire["topic"], serde_json::json!("editor.document"));
    let decoded = serde_json::from_value::<EditorMessageDelivery>(wire.clone())
        .expect("valid complete delivery should deserialize");
    assert_eq!(serde_json::to_value(&decoded).unwrap(), wire);

    for invalid in [
        "",
        "editor",
        "editor..document",
        "Editor.document",
        "editor.例",
    ] {
        wire["topic"] = serde_json::json!(invalid);
        assert!(
            serde_json::from_value::<EditorMessageDelivery>(wire.clone()).is_err(),
            "delivery accepted invalid topic {invalid:?}"
        );
    }
    wire["topic"] = serde_json::json!(7);
    assert!(serde_json::from_value::<EditorMessageDelivery>(wire).is_err());
}
