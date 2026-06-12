use crate::asset::{AssetLoadState, Handle, ProjectAssetManager, TextureAsset};
use crate::core::resource::{
    ResourceDiagnostic, ResourceId, ResourceKind, ResourceRecord, ResourceState,
};

#[test]
fn failed_asset_exposes_failure_reason_through_facade() {
    let manager = ProjectAssetManager::default();
    let locator =
        crate::asset::AssetUri::parse("res://textures/failing.png").expect("valid asset locator");
    let id = ResourceId::from_locator(&locator);
    let record = ResourceRecord::new(id, ResourceKind::Texture, locator)
        .with_state(ResourceState::Error)
        .with_diagnostics(vec![ResourceDiagnostic::error("texture decode failed")]);

    manager.resource_manager().register_record(record);
    let handle = Handle::<TextureAsset>::new(id);

    assert_eq!(manager.load_state(handle), AssetLoadState::Failed);
    assert_eq!(
        manager.failure_reason(handle).as_deref(),
        Some("texture decode failed")
    );
    assert_eq!(
        manager
            .assets::<TextureAsset>()
            .failure_reason(handle)
            .as_deref(),
        Some("texture decode failed")
    );
}
