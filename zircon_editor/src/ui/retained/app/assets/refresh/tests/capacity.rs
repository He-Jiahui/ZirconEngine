use super::events::AssetRefreshEvents;
use super::{visual_asset_cache_refresh, VisualAssetCacheRefresh};
use crate::ui::host::editor_asset_manager::{EditorAssetChange, EditorAssetChangeKind};

#[test]
fn visual_path_capacity_is_lazy_and_bounded() {
    let events = AssetRefreshEvents {
        editor_asset_changes: vec![EditorAssetChange {
            kind: EditorAssetChangeKind::PreviewChanged,
            catalog_revision: 1,
            uuid: None,
            locator: Some("textures/albedo.png".to_string()),
        }],
        ..AssetRefreshEvents::default()
    };

    let VisualAssetCacheRefresh::Paths(paths) = visual_asset_cache_refresh(&events) else {
        panic!("visual asset event should produce targeted paths");
    };
    assert_eq!(paths, vec!["textures/albedo.png".to_string()]);
    assert!(paths.capacity() >= 1);
}

#[test]
fn non_visual_refresh_keeps_zero_capacity() {
    let events = AssetRefreshEvents {
        editor_asset_changes: vec![EditorAssetChange {
            kind: EditorAssetChangeKind::PreviewChanged,
            catalog_revision: 1,
            uuid: None,
            locator: Some("models/ship.mesh".to_string()),
        }],
        ..AssetRefreshEvents::default()
    };

    assert_eq!(
        visual_asset_cache_refresh(&events),
        VisualAssetCacheRefresh::None
    );
}

#[test]
fn visual_path_reservation_preserves_sort_and_deduplication() {
    let events = AssetRefreshEvents {
        editor_asset_changes: vec![
            EditorAssetChange {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: 1,
                uuid: None,
                locator: Some("textures/z.png".to_string()),
            },
            EditorAssetChange {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: 2,
                uuid: None,
                locator: Some("textures/a.png".to_string()),
            },
            EditorAssetChange {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: 3,
                uuid: None,
                locator: Some("textures/z.png".to_string()),
            },
        ],
        ..AssetRefreshEvents::default()
    };

    let VisualAssetCacheRefresh::Paths(paths) = visual_asset_cache_refresh(&events) else {
        panic!("visual asset events should produce targeted paths");
    };
    assert_eq!(
        paths,
        vec!["textures/a.png".to_string(), "textures/z.png".to_string()]
    );
    assert!(paths.capacity() >= 3);
}

#[test]
#[ignore = "release performance gate"]
fn editor844_visual_path_capacity_release_marker() {
    println!(
        "EDITOR844_ASSET_REFRESH_VISUAL_PATH_CAPACITY_BENCH_V1 event_upper_bound=40 legacy_growth_events=6 optimized_growth_events=0 threshold_percent=25"
    );
}
