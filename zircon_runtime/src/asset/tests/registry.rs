use crate::core::resource::{ResourceManager, ResourceRecord};

use crate::asset::{AssetId, AssetKind, AssetUri};

#[test]
fn asset_registry_tracks_add_update_delete_and_rename() {
    let manager = ResourceManager::new();
    let original_uri = AssetUri::parse("res://materials/grid.zmaterial").unwrap();
    let renamed_uri = AssetUri::parse("res://materials/grid_renamed.zmaterial").unwrap();
    let asset_id = AssetId::new();

    let original = ResourceRecord::new(asset_id, AssetKind::Material, original_uri.clone())
        .with_source_hash("source-a")
        .with_importer_version(1)
        .with_config_hash("config-a");
    manager.register_record(original.clone()).unwrap();

    assert_eq!(manager.registry().get(asset_id).unwrap(), &original);
    assert_eq!(
        manager.registry().get_by_locator(&original_uri).unwrap(),
        &original
    );

    let updated = ResourceRecord::new(asset_id, AssetKind::Material, original_uri.clone())
        .with_source_hash("source-b")
        .with_importer_version(1)
        .with_config_hash("config-a");
    manager.register_record(updated.clone()).unwrap();

    assert_eq!(manager.registry().get(asset_id).unwrap(), &updated);
    assert_eq!(
        manager.registry().get_by_locator(&original_uri).unwrap(),
        &updated
    );

    let renamed = manager.rename(&original_uri, renamed_uri.clone()).unwrap();
    assert_eq!(renamed.id(), asset_id);
    assert!(manager.registry().get_by_locator(&original_uri).is_none());
    assert_eq!(
        manager
            .registry()
            .get_by_locator(&renamed_uri)
            .unwrap()
            .primary_locator(),
        &renamed_uri
    );

    let removed = manager.remove_by_locator(&renamed_uri).unwrap().unwrap();
    assert_eq!(removed.id(), asset_id);
    assert!(manager.registry().get(asset_id).is_none());
}
