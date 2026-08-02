use super::*;

#[test]
fn assets_insert_remove_and_project_manager_helpers_use_typed_facade() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture_locator = locator("res://textures/inserted.png");
    let texture_record = record("res://textures/inserted.png", ResourceKind::Texture);
    let texture_id = texture_record.id;
    let texture_events = manager.subscribe_asset_events::<TextureAsset>();

    let handle = manager
        .assets::<TextureAsset>()
        .insert(texture_record, texture_asset("res://textures/inserted.png"))
        .expect("inserted texture handle");

    assert_eq!(handle.id(), texture_id);
    assert_eq!(
        manager
            .handle::<TextureAsset>(&texture_locator)
            .unwrap()
            .id(),
        texture_id
    );
    assert_eq!(
        manager.recursive_dependency_load_state(handle),
        RecursiveDependencyLoadState::Loaded
    );

    let added = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("texture added event");
    assert!(matches!(added, AssetEvent::Added { .. }));
    assert_eq!(added.handle().id(), texture_id);

    assert!(
        manager
            .assets::<ShaderAsset>()
            .remove_by_locator(&texture_locator)
            .is_none()
    );
    assert_eq!(
        resource_manager.registry().get(texture_id).unwrap().kind,
        ResourceKind::Texture
    );

    let removed = manager
        .assets::<TextureAsset>()
        .remove_by_locator(&texture_locator)
        .expect("removed texture record");
    assert_eq!(removed.id, texture_id);
    assert!(resource_manager.registry().get(texture_id).is_none());
}

#[test]
fn project_asset_manager_load_returns_typed_handle_and_state() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let texture_locator = texture_record.primary_locator.clone();
    let texture_id = texture_record.id;
    resource_manager.register_ready(texture_record, texture_asset("res://textures/checker.png"));

    let handle = manager
        .load::<TextureAsset>(&texture_locator)
        .expect("typed texture load");

    assert_eq!(handle.id(), texture_id);
    assert_eq!(
        manager.assets::<TextureAsset>().get(handle).unwrap().width,
        1
    );
    assert_eq!(manager.load_state(handle), AssetLoadState::Loaded);
    assert!(manager.load::<ShaderAsset>(&texture_locator).is_err());

    let pending_record = ResourceRecord::new(
        ResourceId::from_locator(&locator("res://materials/pending.zmaterial")),
        ResourceKind::Material,
        locator("res://materials/pending.zmaterial"),
    )
    .with_state(ResourceState::Pending);
    let pending_id = pending_record.id;
    resource_manager.register_record(pending_record);
    assert_eq!(
        manager.asset_load_state_by_id::<MaterialAsset>(pending_id),
        AssetLoadState::Loading
    );
}

#[test]
fn project_asset_manager_load_accepts_v2_ui_payload_under_ui_layout_kind() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let ui_record = record("res://ui/panel.zui", ResourceKind::UiLayout);
    let ui_locator = ui_record.primary_locator.clone();
    let ui_id = ui_record.id;
    resource_manager.register_ready(ui_record, ui_v2_view_asset());

    let handle = manager
        .load::<UiV2ViewAsset>(&ui_locator)
        .expect("typed v2 ui view load");

    assert_eq!(handle.id(), ui_id);
    assert_eq!(
        manager
            .assets::<UiV2ViewAsset>()
            .get(handle)
            .unwrap()
            .document
            .asset
            .id,
        "runtime.ui.panel"
    );
    assert!(manager.load::<UiLayoutAsset>(&ui_locator).is_err());
}
