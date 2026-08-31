use super::*;

#[test]
fn list_view_selected_asset_row_does_not_impersonate_keyboard_focus() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=4).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let table = find_node(&nodes, "AssetBrowserAssetTablePanel");
    let row = find_node(&nodes, "WorkbenchAssetBrowserAssetRow01");
    let next_row = find_node(&nodes, "WorkbenchAssetBrowserAssetRow02");
    let summary = find_node(&nodes, "AssetBrowserContentPreviewCard");

    assert!(table.frame.height > 0.0);
    assert!(row.frame.height > 0.0);
    assert!(summary.frame.height > 0.0);
    assert!(row.selected);
    assert!(
        !row.focused,
        "Asset Browser list selection should use selected row affordance without forcing focus semantics"
    );
    assert!(!next_row.selected);
    assert!(!next_row.focused);
    assert_eq!(
        row.options.row_data(0).as_deref(),
        Some("Asset_01.mesh"),
        "list rows should keep readable asset names instead of generic category aliases"
    );
    assert!(row.text.as_str().contains("Asset_01.mesh"));
}

#[test]
fn list_view_projects_every_catalog_asset_into_the_clipped_table() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: (1..=7).map(|index| asset_item(index, false)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let table = find_node(&nodes, "AssetBrowserAssetTablePanel");
    let header = find_node(&nodes, "WorkbenchAssetBrowserTableHeader");
    let seventh = find_node(&nodes, "WorkbenchAssetBrowserAssetRow07");

    assert_control_absent(&nodes, "WorkbenchAssetBrowserAssetRow08");
    assert_eq!(table.value_number, 7.0 * 28.0);
    assert_eq!(seventh.frame.height, 28.0);
    assert_eq!(
        seventh.frame.y,
        header.frame.y + header.frame.height + 6.0 * 28.0
    );
    assert!(table.frame.height > header.frame.height);
    assert!(seventh.text.as_str().contains("Asset_07.mesh"));
}

#[test]
fn list_view_summary_keeps_file_like_selected_name_single_line() {
    let mut asset = asset_item(1, true);
    asset.display_name = "workbench_page_chrome.zui".to_string();
    asset.file_name = "workbench_page_chrome.zui".to_string();
    asset.extension = "zui".to_string();
    asset.kind = ResourceKind::UiLayout;
    asset.asset_type =
        crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::UiLayout,
        );
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset].into(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let name = find_node(&nodes, "AssetBrowserContentPreviewName");
    let continuation = find_node(&nodes, "AssetBrowserContentPreviewNameContinuation");
    let type_badge = find_node(&nodes, "AssetBrowserContentPreviewTypeBadge");

    assert_eq!(name.text.as_str(), "workbench_page_chrome.zui");
    assert_eq!(continuation.text.as_str(), "");
    assert_eq!(continuation.frame.height, 0.0);
    assert!(
        name.frame.height >= name.font_size * 1.35,
        "summary title slot should leave baseline-safe room for underscores and descenders: name={:?}",
        name
    );
    assert!(
        type_badge.frame.y - name.frame.y < 24.0,
        "file-like summary titles should stay in a compact one-line detail rhythm: name={:?}, badge={:?}",
        name.frame,
        type_badge.frame
    );
}

#[test]
fn list_view_summary_uses_square_icon_slot_and_compact_field_row() {
    let mut asset = asset_item(1, true);
    asset.display_name = "workbench_page_chrome.zui".to_string();
    asset.file_name = "workbench_page_chrome.zui".to_string();
    asset.extension = "zui".to_string();
    asset.kind = ResourceKind::UiLayout;
    asset.asset_type =
        crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::UiLayout,
        );
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset].into(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let summary = find_node(&nodes, "AssetBrowserContentPreviewCard");
    let visual = find_node(&nodes, "AssetBrowserContentPreviewVisual");
    let name = find_node(&nodes, "AssetBrowserContentPreviewName");
    let type_badge = find_node(&nodes, "AssetBrowserContentPreviewTypeBadge");
    let type_label = find_node(&nodes, "AssetBrowserContentPreviewType");
    let state = find_node(&nodes, "AssetBrowserContentPreviewState");
    let revision = find_node(&nodes, "AssetBrowserContentPreviewRevision");

    assert_eq!(visual.surface_variant.as_str(), "asset-preview-visual");
    assert_eq!(visual.component_role.as_str(), "asset-thumbnail-visual");
    assert_eq!(visual.component_variant.as_str(), "asset-ui-layout");
    assert_eq!(
        visual.frame.width, visual.frame.height,
        "selected summary visual should be a square asset icon slot, not a wide empty preview pill"
    );
    assert!(
        visual.frame.width <= summary.frame.height,
        "summary icon slot should be derived from the compact summary height: visual={:?}, summary={:?}",
        visual.frame,
        summary.frame
    );
    assert!(
        name.frame.x - (visual.frame.x + visual.frame.width) <= 12.0,
        "summary title should sit close to the icon slot like a dense Content Browser field row: visual={:?}, name={:?}",
        visual.frame,
        name.frame
    );
    assert!(
        name.frame.height >= name.font_size * 1.35,
        "summary title should keep a baseline-safe text slot before compact composites build on it: {:?}",
        name
    );
    assert_eq!(type_badge.frame.y, state.frame.y);
    assert_eq!(state.frame.y, revision.frame.y);
    assert_eq!(type_label.text.as_str(), "UI Layout");
    assert!(
        type_badge.frame.width > type_label.frame.width && type_badge.frame.width <= 76.0,
        "summary type badge should use a readable label and adapt to its text frame: badge={:?}, label={:?}",
        type_badge.frame,
        type_label.frame
    );
    assert!(
        type_badge.frame.y > name.frame.y
            && type_badge.frame.y < visual.frame.y + visual.frame.height,
        "type/state/revision row should stay inside the icon-slot vertical rhythm: badge={:?}, visual={:?}, name={:?}",
        type_badge.frame,
        visual.frame,
        name.frame
    );
}
