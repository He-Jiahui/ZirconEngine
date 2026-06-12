use std::time::Duration;

use crate::asset::{AssetEvent, AssetLoadState, Assets, Handle, TextureAsset};
use crate::core::resource::{
    ResourceDiagnostic, ResourceKind, ResourceManager, ResourceState, TextureMarker,
};

use super::{record, texture_asset};

#[test]
fn hot_reload_transitions_through_reloading_state_and_emits_modified_event() {
    let manager = ResourceManager::new();
    let textures = Assets::<TextureAsset>::new(manager.clone());
    let texture_events = textures.subscribe_events();
    let texture_record = record("res://textures/reload-success.png", ResourceKind::Texture);
    let texture_id = texture_record.id;
    let texture_handle = manager
        .register_ready(
            texture_record,
            texture_asset("res://textures/reload-success.png"),
        )
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("texture handle");

    assert!(matches!(
        texture_events.recv_timeout(Duration::from_secs(1)).unwrap(),
        AssetEvent::Added { .. }
    ));

    let reloading = manager
        .start_reload(texture_id, Vec::new())
        .expect("start reload");
    assert_eq!(reloading.state, ResourceState::Reloading);
    assert_eq!(
        textures.load_state(texture_handle),
        AssetLoadState::Reloading
    );
    assert!(matches!(
        texture_events.recv_timeout(Duration::from_secs(1)).unwrap(),
        AssetEvent::Modified { .. }
    ));

    manager.register_ready(
        record("res://textures/reload-success.png", ResourceKind::Texture).with_source_hash("v2"),
        texture_asset("res://textures/reload-success.png"),
    );
    assert_eq!(textures.load_state(texture_handle), AssetLoadState::Loaded);
    assert!(matches!(
        texture_events.recv_timeout(Duration::from_secs(1)).unwrap(),
        AssetEvent::Modified { .. }
    ));
}

#[test]
fn reload_failure_emits_reload_failed_event_and_lands_failed_state() {
    let manager = ResourceManager::new();
    let textures = Assets::<TextureAsset>::new(manager.clone());
    let texture_events = textures.subscribe_events();
    let texture_record = record("res://textures/reload-failed.png", ResourceKind::Texture);
    let texture_id = texture_record.id;
    let texture_handle = manager
        .register_ready(
            texture_record,
            texture_asset("res://textures/reload-failed.png"),
        )
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("texture handle");

    assert!(matches!(
        texture_events.recv_timeout(Duration::from_secs(1)).unwrap(),
        AssetEvent::Added { .. }
    ));

    manager.start_reload(texture_id, Vec::new()).unwrap();
    assert!(matches!(
        texture_events.recv_timeout(Duration::from_secs(1)).unwrap(),
        AssetEvent::Modified { .. }
    ));

    manager.fail_reload(
        texture_id,
        vec![ResourceDiagnostic::error("hot reload decode failed")],
    );
    assert_eq!(textures.load_state(texture_handle), AssetLoadState::Failed);
    assert_eq!(
        textures.failure_reason(texture_handle).as_deref(),
        Some("hot reload decode failed")
    );
    assert!(matches!(
        texture_events.recv_timeout(Duration::from_secs(1)).unwrap(),
        AssetEvent::ReloadFailed { .. }
    ));
}
