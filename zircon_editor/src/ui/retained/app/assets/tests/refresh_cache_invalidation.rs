use super::{
    path_is_sprite_atlas_source, path_is_visual_asset, visual_asset_cache_refresh,
    AssetRefreshEvents, VisualAssetCacheRefresh,
};
use zircon_runtime::resource::ResourceEvent;
use zircon_runtime_interface::resource::{ResourceEventKind, ResourceId, ResourceKind};

#[test]
fn visual_asset_detection_ignores_non_image_resource_churn() {
    assert!(!path_is_visual_asset("models/cube.mesh"));
    assert!(!path_is_visual_asset("scenes/main.scene.toml"));
    assert!(!path_is_visual_asset(".zircon/cache/assets/chunks/01.bin"));
}

#[test]
fn visual_asset_detection_accepts_supported_image_sources() {
    assert!(path_is_visual_asset("icons/Save.SVG"));
    assert!(path_is_visual_asset("textures/albedo.png#preview"));
}

#[test]
fn sprite_atlas_products_keep_the_conservative_full_invalidation_path() {
    assert!(path_is_sprite_atlas_source(
        ".zircon/cache/editor-sprite-atlases/icons.png"
    ));
    assert!(path_is_sprite_atlas_source(
        "editor-sprite-atlases/icons.toml"
    ));
    assert!(!path_is_sprite_atlas_source("assets/icons/save.svg"));
}

#[test]
fn resource_stream_lag_reconciles_resident_sources_instead_of_clearing_all_caches() {
    let events = AssetRefreshEvents {
        resource_generation_lagged: true,
        ..AssetRefreshEvents::default()
    };

    assert_eq!(
        visual_asset_cache_refresh(&events),
        VisualAssetCacheRefresh::Reconcile
    );
}

#[test]
fn unlocated_runtime_texture_does_not_invalidate_file_backed_visual_assets() {
    let events = AssetRefreshEvents {
        resource_changes: vec![ResourceEvent {
            kind: ResourceEventKind::Updated,
            resource_kind: ResourceKind::Texture,
            id: ResourceId::new(),
            locator: None,
            previous_locator: None,
            revision: 1,
        }],
        ..AssetRefreshEvents::default()
    };

    assert_eq!(
        visual_asset_cache_refresh(&events),
        VisualAssetCacheRefresh::None
    );
}

#[test]
fn resource_stream_lag_reconciles_even_with_unlocated_runtime_texture_churn() {
    let events = AssetRefreshEvents {
        resource_changes: vec![ResourceEvent {
            kind: ResourceEventKind::Updated,
            resource_kind: ResourceKind::Texture,
            id: ResourceId::new(),
            locator: None,
            previous_locator: None,
            revision: 1,
        }],
        resource_generation_lagged: true,
        ..AssetRefreshEvents::default()
    };

    assert_eq!(
        visual_asset_cache_refresh(&events),
        VisualAssetCacheRefresh::Reconcile
    );
}
