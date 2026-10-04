use super::*;

#[test]
fn default_layout_uses_the_current_version_shell_and_roundtrips() {
    let layout = WorkbenchLayout::default();

    let encoded = encode_default_layout_value(layout.clone()).unwrap();
    let header = &encoded["$zircon"]["header"];

    assert_eq!(header["schema_id"], DefaultLayoutDocument::SCHEMA.as_str());
    assert_eq!(header["schema_version"], DefaultLayoutDocument::VERSION);
    assert_eq!(decode_default_layout_value(encoded).unwrap(), layout);
}

#[test]
fn raw_legacy_layout_is_rejected_instead_of_becoming_a_second_reader() {
    let legacy = serde_json::to_value(WorkbenchLayout::default()).unwrap();
    let legacy_named = serde_json::to_value(BTreeMap::<String, WorkbenchLayout>::new()).unwrap();
    let legacy_page = serde_json::to_value(LayoutPresetPersistenceStore::default()).unwrap();

    let error = decode_default_layout_value(legacy).unwrap_err();

    assert!(matches!(
        error,
        LayoutPersistenceDocumentError::Decode(LoadError::Migration(_))
    ));
    assert!(matches!(
        decode_named_layout_presets_value(legacy_named),
        Err(LayoutPersistenceDocumentError::Decode(
            LoadError::Migration(_)
        ))
    ));
    assert!(matches!(
        decode_page_layout_presets_value(legacy_page),
        Err(LayoutPersistenceDocumentError::Decode(
            LoadError::Migration(_)
        ))
    ));
}

#[test]
fn layout_payload_kinds_have_distinct_schemas() {
    let default_layout = encode_default_layout_value(WorkbenchLayout::default()).unwrap();
    let mut named_presets = BTreeMap::new();
    named_presets.insert("authoring".to_string(), WorkbenchLayout::default());
    let named = encode_named_layout_presets_value(named_presets.clone()).unwrap();
    let page_store = LayoutPresetPersistenceStore::default();
    let page = encode_page_layout_presets_value(page_store.clone()).unwrap();

    let schema = |value: &Value| value["$zircon"]["header"]["schema_id"].clone();
    assert_ne!(schema(&default_layout), schema(&named));
    assert_ne!(schema(&default_layout), schema(&page));
    assert_ne!(schema(&named), schema(&page));
    assert!(decode_default_layout_value(named).is_err());
    assert_eq!(
        decode_named_layout_presets_value(
            encode_named_layout_presets_value(named_presets.clone()).unwrap()
        )
        .unwrap(),
        named_presets
    );
    assert_eq!(decode_page_layout_presets_value(page).unwrap(), page_store);
}
