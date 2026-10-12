use super::*;

#[test]
fn field_type_normalization_borrows_ascii_aliases_and_preserves_qualified_types() {
    for (type_name, expected) in [
        ("I64", "number"),
        ("BoOlEaN", "bool"),
        ("LinearCOLOR", "color"),
        ("EditorENUM", "enum"),
        ("TextureASSET", "asset_reference"),
        ("GpuRESOURCE", "asset_reference"),
        ("AnimationCURVE", "curve"),
        ("OpaqueCustomRecord", "OpaqueCustomRecord"),
    ] {
        assert_eq!(normalize_field_type_name(type_name), expected);
    }

    let editors = FieldEditorContainer::builtin();
    assert!(editors.definition("Plugin.TextureAsset").is_none());
    assert!(editors.definition("plugin::LinearColor").is_none());
}
