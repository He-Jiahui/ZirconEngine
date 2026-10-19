//! typed handle 和事件是 ResourceManager 记录的受限视图；订阅者只应收到本类型的完整生命周期事件。

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
        .unwrap()
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("texture handle");
    manager
        .register_ready(material_record, material_asset("builtin://shader/pbr.wgsl"))
        .unwrap();

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

// Removed 发出时记录已从注册表消失，过滤器仍须依据事件携带的 kind 保留本类型通知。
#[test]
fn typed_asset_events_filter_by_asset_kind_including_removed_events() {
    let manager = ResourceManager::new();
    let texture_events = Assets::<TextureAsset>::new(manager.clone()).subscribe_events();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let shader_record = record("res://shaders/pbr.wgsl", ResourceKind::Shader);
    let texture_locator = texture_record.primary_locator.clone();
    let shader_locator = shader_record.primary_locator.clone();
    let texture_id = texture_record.id;

    manager
        .register_ready(texture_record, texture_asset("res://textures/checker.png"))
        .unwrap();
    manager
        .register_ready(shader_record, shader_asset("res://shaders/pbr.wgsl"))
        .unwrap();
    manager.remove_by_locator(&shader_locator).unwrap();
    manager.remove_by_locator(&texture_locator).unwrap();

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

    manager
        .register_ready(texture_record, texture_asset("res://textures/order.png"))
        .unwrap();
    manager
        .rename(&original_locator, renamed_locator.clone())
        .expect("rename texture");
    manager.start_reload(texture_id, Vec::new()).unwrap();
    manager
        .fail_reload(texture_id, vec![ResourceDiagnostic::error("reload failed")])
        .unwrap();
    manager.remove_by_locator(&renamed_locator).unwrap();

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

fn concrete_identity_icon() -> crate::asset::UiIconAsset {
    crate::asset::UiIconAsset {
        source: crate::asset::UiIconSource {
            kind: crate::asset::UiIconSourceKind::Svg,
            text: Some("<svg/>".into()),
            uri: None,
        },
        default_size: 16.0,
        semantic_id: "test.identity".into(),
    }
}

#[test]
fn same_marker_contains_tracks_declared_payload_type_after_last_lease_release() {
    use crate::asset::{ImportedAsset, UiIconAsset, UiStyleAsset, UiThemeAsset};
    let manager = ResourceManager::new();
    let texture = record("res://identity/texture", ResourceKind::Texture);
    let icon = record("res://identity/icon", ResourceKind::Texture);
    let style = record("res://identity/style", ResourceKind::UiStyle);
    let theme = record("res://identity/theme", ResourceKind::UiStyle);
    let style_asset = UiStyleAsset::from_toml_str(
        r#"
[asset]
kind = "style"
id = "test.identity.style"
version = 3
[imports]
widgets = []
styles = []
[tokens]
[components]
[[stylesheets]]
id = "identity"
[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Identity" } }
"#,
    )
    .unwrap();
    let theme_asset = UiThemeAsset {
        document: Default::default(),
    };
    assert_ne!(
        ImportedAsset::UiStyle(style_asset.clone()).payload_type(),
        ImportedAsset::UiTheme(theme_asset.clone()).payload_type()
    );
    assert_ne!(
        ImportedAsset::Texture(texture_asset("res://identity/texture")).payload_type(),
        ImportedAsset::UiIcon(concrete_identity_icon()).payload_type()
    );
    manager
        .register_ready(texture.clone(), texture_asset("res://identity/texture"))
        .unwrap();
    manager
        .register_ready(icon.clone(), concrete_identity_icon())
        .unwrap();
    manager.register_ready(style.clone(), style_asset).unwrap();
    manager.register_ready(theme.clone(), theme_asset).unwrap();
    let textures = Assets::<TextureAsset>::new(manager.clone());
    let icons = Assets::<UiIconAsset>::new(manager.clone());
    let styles = Assets::<UiStyleAsset>::new(manager.clone());
    let themes = Assets::<UiThemeAsset>::new(manager.clone());
    assert!(textures.contains(Handle::new(texture.id)));
    assert!(!textures.contains(Handle::new(icon.id)));
    assert!(icons.contains(Handle::new(icon.id)));
    assert!(!icons.contains(Handle::new(texture.id)));
    assert!(styles.contains(Handle::new(style.id)));
    assert!(!styles.contains(Handle::new(theme.id)));
    assert!(themes.contains(Handle::new(theme.id)));
    assert!(!themes.contains(Handle::new(style.id)));
    let retained_snapshot = manager.registry().clone();
    let lease = icons.acquire(Handle::new(icon.id)).unwrap();
    drop(lease);
    assert_eq!(manager.ref_count(icon.id), Some(0));
    assert!(icons.get(Handle::new(icon.id)).is_none());
    assert!(icons.contains(Handle::new(icon.id)));
    assert!(!textures.contains(Handle::new(icon.id)));
    manager.remove_by_locator(&icon.primary_locator).unwrap();
    assert!(!icons.contains(Handle::new(icon.id)));
    assert!(retained_snapshot.contains_payload::<UiIconAsset>(icon.id));
}

#[test]
fn same_marker_events_keep_published_type_through_remove_and_id_reuse() {
    use crate::asset::UiIconAsset;
    let manager = ResourceManager::new();
    let textures = Assets::<TextureAsset>::new(manager.clone());
    let icons = Assets::<UiIconAsset>::new(manager.clone());
    let texture_events = textures.subscribe_events();
    let icon_events = icons.subscribe_events();
    let original = record("res://identity/reused", ResourceKind::Texture);
    manager
        .register_ready(original.clone(), texture_asset("res://identity/reused"))
        .unwrap();
    assert!(matches!(
        texture_events.try_recv().unwrap(),
        AssetEvent::Added { .. }
    ));
    assert!(icon_events.try_recv().is_err());
    manager.start_reload(original.id, Vec::new()).unwrap();
    manager
        .remove_by_locator(&original.primary_locator)
        .unwrap();
    // Do not consume the old events until the ID belongs to another concrete payload.
    manager
        .register_ready(original.clone(), concrete_identity_icon())
        .unwrap();
    assert!(matches!(
        texture_events.try_recv().unwrap(),
        AssetEvent::Modified { .. }
    ));
    let removed = texture_events.try_recv().unwrap();
    assert!(matches!(removed, AssetEvent::Removed { .. }));
    assert_eq!(removed.handle().id(), original.id);
    assert!(texture_events.try_recv().is_err());
    let added = icon_events.try_recv().unwrap();
    assert!(matches!(added, AssetEvent::Added { .. }));
    assert_eq!(added.handle().id(), original.id);
    assert!(icon_events.try_recv().is_err());
    assert!(icons.contains(Handle::new(original.id)));
    assert!(!textures.contains(Handle::new(original.id)));
}
