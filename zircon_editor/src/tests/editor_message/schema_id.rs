use crate::core::editor_message::{
    EditorMessageSchemaId, EditorMessageSchemaIdError, MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES,
};

#[test]
fn schema_id_builders_confine_builtin_and_plugin_namespaces() {
    let editor = EditorMessageSchemaId::editor("world_fact.v1").unwrap();
    let plugin = EditorMessageSchemaId::plugin("weather.editor", "telemetry.v1").unwrap();

    assert_eq!(editor.as_str(), "zircon.editor.world_fact.v1");
    assert_eq!(plugin.as_str(), "zircon.plugin.weather.editor.telemetry.v1");
}

#[test]
fn schema_id_parser_rejects_unowned_or_malformed_namespaces() {
    for invalid in [
        "",
        "zircon.editor",
        "zircon.plugin.weather",
        "editor.debug-text",
        "plugin.weather.telemetry.v1",
        "zircon.runtime.world_fact.v1",
        "zircon.editor.world..v1",
        "zircon.editor.WorldFact.v1",
    ] {
        assert!(
            EditorMessageSchemaId::parse(invalid).is_err(),
            "schema id `{invalid}` must be rejected"
        );
    }
}

#[test]
fn schema_id_parser_enforces_the_named_protocol_size_limit() {
    let local = "a".repeat(MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES);
    let error = EditorMessageSchemaId::editor(local).unwrap_err();

    assert!(matches!(
        error,
        EditorMessageSchemaIdError::TooLong {
            max_bytes: MAX_EDITOR_MESSAGE_SCHEMA_ID_BYTES,
            ..
        }
    ));
}

#[test]
fn schema_id_serde_roundtrip_validates_untrusted_input() {
    let schema_id = EditorMessageSchemaId::plugin("weather", "forecast.v1").unwrap();
    let encoded = serde_json::to_string(&schema_id).unwrap();

    assert_eq!(
        serde_json::from_str::<EditorMessageSchemaId>(&encoded).unwrap(),
        schema_id
    );
    assert!(serde_json::from_str::<EditorMessageSchemaId>(r#""zircon.plugin.weather""#).is_err());
    assert!(serde_json::from_str::<EditorMessageSchemaId>(r#""foreign.schema.v1""#).is_err());
}
