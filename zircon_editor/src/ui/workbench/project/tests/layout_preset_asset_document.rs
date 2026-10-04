use super::*;

#[test]
fn project_layout_preset_uses_the_current_version_shell_and_roundtrips() {
    let layout = WorkbenchLayout::default();

    let encoded = encode_layout_preset_asset_document(&layout).unwrap();

    assert!(encoded.contains(LayoutPresetAssetDocument::SCHEMA.as_str()));
    assert_eq!(
        decode_layout_preset_asset_document(encoded.as_bytes()).unwrap(),
        layout
    );
}

#[test]
fn unversioned_project_layout_preset_is_rejected() {
    let legacy = serde_json::to_vec(&LayoutPresetAssetDocument {
        workbench: WorkbenchLayout::default(),
    })
    .unwrap();

    assert!(matches!(
        decode_layout_preset_asset_document(&legacy),
        Err(LoadError::MissingTextEnvelope { .. })
    ));
}
