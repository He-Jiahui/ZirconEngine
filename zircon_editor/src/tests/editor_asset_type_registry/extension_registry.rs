use crate::core::asset::{
    AssetToolkitDescriptor, AssetTypeContribution, AssetTypeId, AssetTypePresentation,
    ThumbnailProviderDescriptor,
};
use crate::core::editor_extension::{EditorExtensionRegistry, EditorExtensionRegistryError};
use crate::core::editor_operation::EditorOperationPath;

#[test]
fn editor_extension_registry_exposes_one_typed_asset_contribution_family() {
    let mut registry = EditorExtensionRegistry::default();
    let material = AssetTypeId::parse("material").unwrap();
    registry
        .register_asset_type_contribution(
            AssetTypeContribution::augment(material.clone()).with_toolkit(
                AssetToolkitDescriptor::new(
                    "editor.material",
                    EditorOperationPath::parse("material.editor.open").unwrap(),
                ),
            ),
        )
        .unwrap();

    let contributions = registry.asset_type_contributions();
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].asset_type(), &material);

    let encoded = serde_json::to_string(&registry).unwrap();
    let decoded: EditorExtensionRegistry = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.asset_type_contributions(), contributions);
}

#[test]
fn one_plugin_cannot_register_two_parallel_contributions_for_the_same_asset_type() {
    let mut registry = EditorExtensionRegistry::default();
    let custom = AssetTypeId::parse("custom.asset").unwrap();
    let contribution = AssetTypeContribution::define(
        custom.clone(),
        AssetTypePresentation::new("Custom Asset", "CUS", "asset-custom", "asset.custom"),
        ThumbnailProviderDescriptor::Icon("asset-custom".to_owned()),
    );
    registry
        .register_asset_type_contribution(contribution.clone())
        .unwrap();

    let error = registry
        .register_asset_type_contribution(contribution)
        .unwrap_err();
    assert!(matches!(
        error,
        EditorExtensionRegistryError::DuplicateContribution {
            kind: "asset type contribution",
            id,
        } if id == custom.as_str()
    ));
}

#[test]
fn legacy_parallel_asset_tables_are_removed_from_the_extension_owner() {
    let source = include_str!("../../core/editor_extension.rs");
    for retired in [
        "asset_editors:",
        "asset_creation_templates:",
        "register_asset_editor(",
        "register_asset_creation_template(",
        "pub fn asset_editors(",
        "pub fn asset_creation_templates(",
        "pub struct AssetEditorDescriptor",
    ] {
        assert!(
            !source.contains(retired),
            "retired extension surface still present: {retired}"
        );
    }
}
