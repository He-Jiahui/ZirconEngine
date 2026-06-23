use super::*;

#[test]
fn typed_handle_roundtrips_and_rejects_kind_mismatch() {
    let id = ResourceId::from_stable_label("res://textures/checker.png");
    let handle = Handle::<TextureAsset>::new(id);
    let raw: ResourceHandle<TextureMarker> = handle.into();
    let untyped: UntypedResourceHandle = handle.into();
    let mesh_id = ResourceId::from_stable_label("res://meshes/triangle.zmesh");
    let mesh_untyped: UntypedResourceHandle = Handle::<MeshAsset>::new(mesh_id).into();

    assert_eq!(handle.id(), id);
    assert_eq!(raw.id(), id);
    assert_eq!(untyped.kind(), ResourceKind::Texture);
    assert_eq!(mesh_untyped.kind(), ResourceKind::Mesh);
    assert_eq!(Handle::<TextureAsset>::try_from(untyped).unwrap().id(), id);
    assert!(Handle::<ShaderAsset>::try_from(untyped).is_err());
}

#[test]
fn assets_get_acquire_release_and_kind_filtering_use_resource_manager_storage() {
    let manager = ResourceManager::new();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let material_record = record("res://materials/grid.zmaterial", ResourceKind::Material);
    let texture_id = texture_record.id;
    let material_id = material_record.id;
    let texture_handle = manager
        .register_ready(texture_record, texture_asset("res://textures/checker.png"))
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("texture handle");
    manager.register_ready(material_record, material_asset("builtin://shader/pbr.wgsl"));

    let textures = Assets::<TextureAsset>::new(manager.clone());
    let wrong_texture_handle = Handle::<TextureAsset>::new(material_id);

    assert!(textures.contains(texture_handle));
    assert!(!textures.contains(wrong_texture_handle));
    assert_eq!(textures.get(texture_handle).unwrap().width, 1);
    assert!(textures.get(wrong_texture_handle).is_none());

    let lease = textures.acquire(texture_handle).expect("texture lease");
    assert_eq!(lease.height, 1);
    assert_eq!(manager.ref_count(texture_id), Some(1));
    drop(lease);
    assert_eq!(manager.ref_count(texture_id), Some(0));
    assert!(textures.get(texture_handle).is_none());
}

#[test]
fn typed_asset_events_filter_by_asset_kind_including_removed_events() {
    let manager = ResourceManager::new();
    let texture_events = Assets::<TextureAsset>::new(manager.clone()).subscribe_events();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let shader_record = record("res://shaders/pbr.wgsl", ResourceKind::Shader);
    let texture_locator = texture_record.primary_locator.clone();
    let shader_locator = shader_record.primary_locator.clone();
    let texture_id = texture_record.id;

    manager.register_ready(texture_record, texture_asset("res://textures/checker.png"));
    manager.register_ready(shader_record, shader_asset("res://shaders/pbr.wgsl"));
    manager.remove_by_locator(&shader_locator);
    manager.remove_by_locator(&texture_locator);

    let added = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("texture added event");
    assert!(matches!(added, AssetEvent::Added { .. }));
    assert_eq!(added.handle().id(), texture_id);

    let removed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("texture removed event");
    assert!(matches!(removed, AssetEvent::Removed { .. }));
    assert_eq!(removed.handle().id(), texture_id);
    assert!(
        texture_events.try_recv().is_err(),
        "shader events must not leak into texture receiver"
    );
}

#[test]
fn typed_asset_events_preserve_rename_reload_and_remove_order() {
    let manager = ResourceManager::new();
    let texture_events = Assets::<TextureAsset>::new(manager.clone()).subscribe_events();
    let original_locator = locator("res://textures/order.png");
    let renamed_locator = locator("res://textures/order-renamed.png");
    let texture_record = record("res://textures/order.png", ResourceKind::Texture);
    let texture_id = texture_record.id;

    manager.register_ready(texture_record, texture_asset("res://textures/order.png"));
    manager
        .rename(&original_locator, renamed_locator.clone())
        .expect("rename texture");
    manager.start_reload(texture_id, Vec::new());
    manager.fail_reload(texture_id, vec![ResourceDiagnostic::error("reload failed")]);
    manager.remove_by_locator(&renamed_locator);

    let added = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("added event");
    assert!(matches!(added, AssetEvent::Added { .. }));

    let renamed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("renamed event");
    match renamed {
        AssetEvent::Renamed {
            handle,
            locator,
            previous_locator,
            ..
        } => {
            assert_eq!(handle.id(), texture_id);
            assert_eq!(locator, Some(renamed_locator.clone()));
            assert_eq!(previous_locator, Some(original_locator));
        }
        other => panic!("expected renamed event, got {other:?}"),
    }

    let modified = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("reload modified event");
    assert!(matches!(modified, AssetEvent::Modified { .. }));

    let failed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("reload failed event");
    assert!(matches!(failed, AssetEvent::ReloadFailed { .. }));

    let removed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("removed event");
    assert!(matches!(removed, AssetEvent::Removed { .. }));
    assert!(texture_events.try_recv().is_err());
}
