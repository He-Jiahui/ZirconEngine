use crate::core::asset::{
    AssetSourceAuthority, AssetSourceKind, AssetSourceWritePolicy, AssetTypeContribution,
    AssetTypeId, AssetTypePresentation, AssetTypeRegistry, AssetTypeRegistryError,
    AssetWriteAccess, ThumbnailProviderDescriptor,
};
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn source_authority_resolves_the_complete_project_and_read_only_matrix() {
    for (locator, expected_kind, expected_access) in [
        (
            "res://materials/hero.material.toml",
            AssetSourceKind::Project,
            AssetWriteAccess::Writable,
        ),
        (
            "package://com.zircon.materials/hero.material.toml",
            AssetSourceKind::Package,
            AssetWriteAccess::ReadOnly,
        ),
        (
            "builtin://materials/default.material.toml",
            AssetSourceKind::Builtin,
            AssetWriteAccess::ReadOnly,
        ),
        (
            "lib://materials/shared.material.toml",
            AssetSourceKind::Library,
            AssetWriteAccess::ReadOnly,
        ),
        (
            "mem://preview/material",
            AssetSourceKind::Transient,
            AssetWriteAccess::ReadOnly,
        ),
    ] {
        let authority =
            AssetSourceAuthority::from_locator_str(AssetSourceWritePolicy::ProjectOnly, locator)
                .unwrap();
        assert_eq!(authority.kind(), expected_kind, "{locator}");
        assert_eq!(authority.write_access(), expected_access, "{locator}");
    }

    let derived = AssetSourceAuthority::derived(AssetSourceWritePolicy::ProjectOnly);
    assert_eq!(derived.kind(), AssetSourceKind::Derived);
    assert_eq!(derived.write_access(), AssetWriteAccess::ReadOnly);

    let read_only_project = AssetSourceAuthority::from_locator_str(
        AssetSourceWritePolicy::ReadOnly,
        "res://materials/locked.material.toml",
    )
    .unwrap();
    assert_eq!(read_only_project.kind(), AssetSourceKind::Project);
    assert_eq!(read_only_project.write_access(), AssetWriteAccess::ReadOnly);
}

#[test]
fn source_write_policy_is_a_serializable_single_owner_definition_field() {
    let custom = AssetTypeId::parse("locked.asset").unwrap();
    let contribution = AssetTypeContribution::define(
        custom.clone(),
        AssetTypePresentation::new("Locked Asset", "LCK", "asset-locked", "asset.locked"),
        ThumbnailProviderDescriptor::SourceImage,
    )
    .with_runtime_kind(ResourceKind::Data)
    .with_source_write_policy(AssetSourceWritePolicy::ReadOnly);
    let encoded = serde_json::to_string(&contribution).unwrap();
    let decoded: AssetTypeContribution = serde_json::from_str(&encoded).unwrap();

    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    registry
        .apply_contribution("plugin.locked", decoded)
        .unwrap();
    assert_eq!(
        registry.get(&custom).unwrap().source_write_policy(),
        AssetSourceWritePolicy::ReadOnly
    );

    let material = AssetTypeId::from_resource_kind(ResourceKind::Material);
    let error = registry
        .apply_contribution(
            "plugin.override",
            AssetTypeContribution::augment(material.clone())
                .with_source_write_policy(AssetSourceWritePolicy::ReadOnly),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        AssetTypeRegistryError::DuplicateFieldOwner {
            asset_type,
            field: "source_write_policy",
            ..
        } if asset_type == material
    ));
}
