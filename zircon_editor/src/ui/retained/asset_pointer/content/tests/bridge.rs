use super::super::AssetPointerContentRoute;
use super::*;

#[test]
fn pane_size_patch_preserves_content_projection() {
    let mut bridge = AssetContentListPointerBridge::new();
    let item_ids = (0..1_024)
        .map(|index| format!("asset-{index}"))
        .collect::<Vec<_>>();
    bridge.sync(
        AssetContentListPointerLayout::for_test(
            UiSize::new(320.0, 180.0),
            AssetContentSurfaceProfile::Browser,
            AssetViewMode::List,
            vec![String::from("res://materials")],
            item_ids.clone(),
        ),
        AssetListPointerState::default(),
    );

    let state_change = bridge.sync_pane_size(UiSize::new(640.0, 360.0));

    assert!(state_change.is_none());
    assert_eq!(
        bridge
            .layout
            .items
            .iter()
            .map(|item| &item.uuid)
            .collect::<Vec<_>>(),
        item_ids.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        bridge.layout.folder_ids,
        vec![String::from("res://materials")]
    );
    assert_eq!(bridge.surface_node_count_for_test(), 2);
}

#[test]
fn payload_only_generation_refresh_preserves_pointer_geometry() {
    let mut bridge = AssetContentListPointerBridge::new();
    let layout = AssetContentListPointerLayout::for_test(
        UiSize::new(320.0, 180.0),
        AssetContentSurfaceProfile::Browser,
        AssetViewMode::List,
        Vec::new(),
        vec!["asset-runtime-material".to_string()],
    );
    let source_items = layout.items.clone();
    assert!(bridge.sync(layout, AssetListPointerState::default()));

    let mut changed = source_items[0].clone();
    changed.display_name.push_str(" updated");
    let next_items = source_items
        .replace_existing_items([changed])
        .expect("payload-only replacement must preserve item identity");
    assert!(source_items.shares_item_identity_with(&next_items));
    assert!(!source_items.shares_items_with(&next_items));

    let authority_generation = bridge.surface_authority_generation_for_test();
    let next_layout = AssetContentListPointerLayout {
        items: next_items.clone(),
        ..bridge.layout.clone()
    };
    assert!(!bridge.sync(next_layout, AssetListPointerState::default()));
    assert_eq!(
        bridge.surface_authority_generation_for_test(),
        authority_generation
    );
    assert!(bridge.layout.items.shares_items_with(&next_items));
}

#[test]
fn route_at_resolves_the_stable_asset_uuid_without_dispatching_input() {
    let mut bridge = AssetContentListPointerBridge::new();
    bridge.sync(
        AssetContentListPointerLayout::for_test(
            UiSize::new(320.0, 180.0),
            AssetContentSurfaceProfile::Browser,
            AssetViewMode::Thumbnail,
            Vec::new(),
            vec!["asset-runtime-material".to_string()],
        ),
        AssetListPointerState::default(),
    );

    assert!(matches!(
        bridge.route_at(UiPoint::new(24.0, 24.0)),
        Some(AssetPointerContentRoute::Item { asset_uuid, .. })
            if asset_uuid == "asset-runtime-material"
    ));
}
