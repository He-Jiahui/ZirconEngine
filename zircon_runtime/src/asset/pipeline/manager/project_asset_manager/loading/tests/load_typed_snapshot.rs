use crate::asset::{AssetUri, ProjectAssetManager, TextureAsset};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};

#[test]
fn texture_snapshot_retains_the_payload_owned_by_its_exact_revision() {
    let manager = ProjectAssetManager::default();
    let uri = AssetUri::parse("res://textures/atomic-snapshot.png").expect("texture uri");
    let id = ResourceId::from_locator(&uri);
    let first_record =
        ResourceRecord::new(id, ResourceKind::Texture, uri.clone()).with_source_hash("white");
    manager
        .assets::<TextureAsset>()
        .insert(
            first_record,
            TextureAsset::new_rgba8(uri.clone(), 1, 1, vec![255, 255, 255, 255]),
        )
        .expect("first texture publication");

    let first = manager
        .load_texture_asset_snapshot(id)
        .expect("first texture snapshot");
    manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Texture, uri.clone()).with_source_hash("black"),
            TextureAsset::new_rgba8(uri, 1, 1, vec![0, 0, 0, 255]),
        )
        .expect("second texture publication");
    let second = manager
        .load_texture_asset_snapshot(id)
        .expect("second texture snapshot");

    assert!(second.revision() > first.revision());
    assert_eq!(first.rgba, vec![255, 255, 255, 255]);
    assert_eq!(second.rgba, vec![0, 0, 0, 255]);
}
