use super::*;

#[test]
fn canonical_current_scene_document_deserializes_as_typed_payload() {
    let scene = DynamicScene::empty();
    let header = crate::scene::dynamic_scene::document::current_dynamic_scene_header();
    let encoded = serde_json::to_string(&SceneDocumentRef {
        envelope: SceneEnvelopeRef {
            header: &header,
            payload: &scene,
        },
    })
    .unwrap();
    let decoded = deserialize(&mut serde_json::Deserializer::from_str(&encoded)).unwrap();

    assert_eq!(decoded, scene);
}

#[test]
fn scene_document_rejects_payload_before_header() {
    let scene = DynamicScene::empty();
    let header = crate::scene::dynamic_scene::document::current_dynamic_scene_header();
    let encoded = format!(
        r#"{{"$zircon":{{"payload":{},"header":{}}}}}"#,
        serde_json::to_string(&scene).unwrap(),
        serde_json::to_string(&header).unwrap(),
    );

    let error = deserialize(&mut serde_json::Deserializer::from_str(&encoded)).unwrap_err();

    assert!(error.to_string().contains("header must precede payload"));
}

#[test]
fn scene_document_rejects_future_header_before_consuming_payload() {
    let scene = DynamicScene::empty();
    let mut header = crate::scene::dynamic_scene::document::current_dynamic_scene_header();
    header.schema_version = header.schema_version.saturating_add(1);
    let encoded = serde_json::to_string(&SceneDocumentRef {
        envelope: SceneEnvelopeRef {
            header: &header,
            payload: &scene,
        },
    })
    .unwrap();

    let error = deserialize(&mut serde_json::Deserializer::from_str(&encoded)).unwrap_err();

    assert!(error.to_string().contains("unsupported format version"));
}
