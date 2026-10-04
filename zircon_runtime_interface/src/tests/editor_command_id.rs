use super::EditorCommandId;

#[test]
fn command_id_golden_grammar_is_shared_at_the_wire_boundary() {
    for valid in [
        "editor.asset.open",
        "plugin_2.graph.compile",
        "view.editor.settings",
    ] {
        let id = EditorCommandId::parse(valid).unwrap();
        assert_eq!(id.as_str(), valid);
        assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{valid}\""));
    }
    for invalid in [
        "",
        "editor",
        "editor.asset",
        ".editor.asset",
        "editor..asset",
        "editor.asset.",
        "Editor.asset.open",
        "editor.asset-open",
    ] {
        assert!(EditorCommandId::parse(invalid).is_err(), "{invalid:?}");
        assert!(serde_json::from_str::<EditorCommandId>(&format!("\"{invalid}\"")).is_err());
    }
}
