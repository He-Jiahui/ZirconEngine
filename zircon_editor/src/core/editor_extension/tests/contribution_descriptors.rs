use super::*;

#[test]
fn menu_descriptor_rejects_the_retired_shortcut_owner() {
    let operation = EditorOperationPath::parse("fixture.editor.command").unwrap();
    let descriptor = EditorMenuItemDescriptor::new(
        EditorCommandMenuPath::builtin(&operation, "tools", &[]),
        operation,
    );
    let mut serialized = serde_json::to_value(descriptor).unwrap();
    serialized
        .as_object_mut()
        .unwrap()
        .insert("shortcut".to_string(), serde_json::json!("Ctrl+Alt+R"));

    assert!(serde_json::from_value::<EditorMenuItemDescriptor>(serialized).is_err());
}

#[test]
fn menu_descriptor_rebuilds_its_stable_path_from_typed_segments() {
    let operation = EditorOperationPath::parse("fixture.editor.command").unwrap();
    let descriptor = EditorMenuItemDescriptor::new(
        EditorCommandMenuPath::builtin(&operation, "tools", &["fixture"]),
        operation,
    );
    let serialized = serde_json::to_value(&descriptor).unwrap();

    assert!(serialized.get("stable_path").is_none());
    let restored: EditorMenuItemDescriptor = serde_json::from_value(serialized).unwrap();
    assert_eq!(restored.path(), "tools/fixture/fixture.editor.command");
}

#[test]
fn menu_descriptor_rejects_the_retired_string_path_payload() {
    let operation = EditorOperationPath::parse("fixture.editor.command").unwrap();
    let descriptor = EditorMenuItemDescriptor::new(
        EditorCommandMenuPath::builtin(&operation, "tools", &["fixture"]),
        operation,
    );
    let mut serialized = serde_json::to_value(descriptor).unwrap();
    serialized.as_object_mut().unwrap().insert(
        "stable_path".to_string(),
        serde_json::json!("Tools/Fixture/Fixture Command"),
    );

    assert!(serde_json::from_value::<EditorMenuItemDescriptor>(serialized).is_err());
}

#[test]
fn descriptor_validation_avoids_short_lived_scan_allocations() {
    let source = include_str!("../contribution_descriptors.rs");
    let menu_collection = ["split('/')", ".collect::<Vec<_>>()"].concat();
    let binding_clone = ["EditorOperationPath::parse", "(binding.clone())"].concat();
    assert!(!source.contains(&menu_collection));
    assert!(!source.contains(&binding_clone));

    let extension_body = source
        .split("fn push_normalized_extension")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn validate_asset_importer").next())
        .expect("extension normalization body should remain available");
    assert!(extension_body.contains("binary_search"));
    assert!(extension_body.contains("extensions.windows(2)"));
}
