use crate::{AssetId, AssetKind, AssetMetadata, AssetRegistry, AssetUri};

#[test]
fn asset_registry_tracks_add_update_delete_and_rename() {
    let mut registry = AssetRegistry::default();
    let original_uri = AssetUri::parse("res://materials/grid.material.toml").unwrap();
    let renamed_uri = AssetUri::parse("res://materials/grid_renamed.material.toml").unwrap();
    let asset_id = AssetId::new();

    let original = AssetMetadata::new(asset_id, AssetKind::Material, original_uri.clone())
        .with_source_hash("source-a")
        .with_importer_version(1)
        .with_config_hash("config-a");
    registry.upsert(original.clone());

    assert_eq!(registry.get(asset_id).unwrap(), &original);
    assert_eq!(registry.get_by_locator(&original_uri).unwrap(), &original);

    let updated = AssetMetadata::new(asset_id, AssetKind::Material, original_uri.clone())
        .with_source_hash("source-b")
        .with_importer_version(1)
        .with_config_hash("config-a");
    registry.upsert(updated.clone());

    assert_eq!(registry.get(asset_id).unwrap(), &updated);
    assert_eq!(registry.get_by_locator(&original_uri).unwrap(), &updated);

    let renamed = registry.rename(&original_uri, renamed_uri.clone()).unwrap();
    assert_eq!(renamed.id(), asset_id);
    assert!(registry.get_by_locator(&original_uri).is_none());
    assert_eq!(
        registry.get_by_locator(&renamed_uri).unwrap().primary_locator(),
        &renamed_uri
    );

    let removed = registry.remove_by_locator(&renamed_uri).unwrap();
    assert_eq!(removed.id(), asset_id);
    assert!(registry.get(asset_id).is_none());
}
