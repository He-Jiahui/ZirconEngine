use super::*;

#[test]
fn asset_tree_disclosure_hit_uses_painted_geometry_and_preserves_drop_route() {
    use super::super::layout::AssetFolderTreePointerRow;
    use crate::ui::retained_host::host_contract::{
        data::FrameRect, paint_theme::current_host_metrics, tree_disclosure_rect_for_indent,
    };
    let mut bridge = AssetFolderTreePointerBridge::new();
    bridge.sync(
        AssetFolderTreePointerLayout {
            pane_size: UiSize::new(240.0, 140.0),
            folder_ids: vec!["root".into(), "branch".into(), "leaf".into(), "tail".into()],
            rows: (0..4)
                .map(|index| AssetFolderTreePointerRow {
                    depth: 1,
                    has_children: index == 1,
                    expanded: true,
                })
                .collect(),
            catalog_revision: 27,
        },
        AssetListPointerState {
            scroll_offset: 32.0,
            ..Default::default()
        },
    );
    let scale = current_host_metrics().scale_factor;
    let row = FrameRect {
        x: ROW_X * scale,
        y: (viewport_y() + ROW_Y) * scale,
        width: 224.0 * scale,
        height: ROW_HEIGHT * scale,
    };
    let disclosure = tree_disclosure_rect_for_indent(1, 0.0, &row);
    let point = UiPoint::new(
        (disclosure.x + disclosure.width * 0.5) / scale,
        (disclosure.y + disclosure.height * 0.5) / scale,
    );
    let clicked = bridge.handle_click(point).unwrap();
    assert_eq!(
        clicked.route,
        Some(super::super::route::AssetPointerTreeRoute::Disclosure {
            row_index: 1,
            folder_id: "branch".into(),
            expanded: false,
            expected_catalog_revision: 27,
        })
    );
    assert!(matches!(
        bridge.route_at(point),
        Some(super::super::route::AssetPointerTreeRoute::Folder { row_index: 1, .. })
    ));
    // Outside the arrow remains ordinary folder selection; leaf rows never expand.
    assert!(matches!(
        bridge
            .handle_click(UiPoint::new(180.0, point.y))
            .unwrap()
            .route,
        Some(super::super::route::AssetPointerTreeRoute::Folder { row_index: 1, .. })
    ));
    assert!(matches!(
        bridge
            .handle_click(UiPoint::new(point.x, point.y + ROW_HEIGHT + ROW_GAP))
            .unwrap()
            .route,
        Some(super::super::route::AssetPointerTreeRoute::Folder { row_index: 2, .. })
    ));
}

#[test]
fn pane_size_patch_preserves_folder_projection() {
    let mut bridge = AssetFolderTreePointerBridge::new();
    let folder_ids = (0..1_024)
        .map(|index| format!("res://folder-{index}"))
        .collect::<Vec<_>>();
    bridge.sync(
        AssetFolderTreePointerLayout {
            pane_size: UiSize::new(240.0, 180.0),
            folder_ids: folder_ids.clone(),
            ..Default::default()
        },
        AssetListPointerState::default(),
    );

    let state_change = bridge.sync_pane_size(UiSize::new(480.0, 360.0));

    assert!(state_change.is_none());
    assert_eq!(bridge.layout.folder_ids, folder_ids);
    assert_eq!(bridge.surface_node_count_for_test(), 2);
}
