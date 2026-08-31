use super::*;

#[test]
fn thumbnail_view_projects_adaptive_compact_grid_cards() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=8).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let content = find_node(&nodes, "AssetBrowserContentPanel");
    let table = find_node(&nodes, "AssetBrowserAssetTablePanel");
    let grid = find_node(&nodes, "AssetBrowserThumbGridPanel");
    let first = find_node(&nodes, "AssetBrowserThumbCard01");
    let second = find_node(&nodes, "AssetBrowserThumbCard02");
    let seventh = find_node(&nodes, "AssetBrowserThumbCard07");
    let first_visual = find_node(&nodes, "AssetBrowserThumbVisual01");
    let first_band = find_node(&nodes, "AssetBrowserThumbInfoBand01");
    let first_marker = find_node(&nodes, "AssetBrowserThumbSelectionMarker01");
    let first_name = find_node(&nodes, "AssetBrowserThumbName01");
    let first_name_continuation = find_node(&nodes, "AssetBrowserThumbNameContinuation01");
    let first_type_badge = find_node(&nodes, "AssetBrowserThumbTypeBadge01");
    let first_type = find_node(&nodes, "AssetBrowserThumbType01");
    let first_meta = find_node(&nodes, "AssetBrowserThumbMeta01");

    assert_eq!(table.frame.height, 0.0);
    assert_eq!(grid.frame.x, content.frame.x);
    assert_eq!(grid.frame.width, content.frame.width);
    assert!(grid.frame.width >= 420.0, "grid width {}", grid.frame.width);
    assert!(grid.frame.height >= 86.0);
    assert!(first.selected);
    assert!(!first.focused);
    assert_eq!(first.surface_variant.as_str(), "asset-thumbnail-card");
    assert_eq!(
        first.border_width, 1.0,
        "selected asset tiles should carry a thin UE-style card border without becoming keyboard-focused"
    );
    assert_eq!(second.surface_variant.as_str(), "asset-thumbnail-card");
    assert_eq!(second.border_width, 0.0);
    assert!(first.frame.width >= 104.0);
    assert!(
        grid.frame.height >= content.frame.height - 26.0,
        "thumbnail grid should reclaim the old inline-summary band: grid={:?}, content={:?}",
        grid.frame,
        content.frame
    );
    let card_aspect = first.frame.height / first.frame.width;
    assert!(
        (1.08..=1.28).contains(&card_aspect),
        "thumbnail tile should keep a Content Browser card proportion instead of a squat label block: card={:?}, aspect={card_aspect}",
        first.frame
    );
    assert!(second.frame.x > first.frame.x);
    assert!(
        seventh.frame.y > first.frame.y,
        "thumbnail grid should show a second Asset Browser row before any details summary: first={:?}, seventh={:?}",
        first.frame,
        seventh.frame
    );
    assert!(first_visual.frame.x > first.frame.x);
    assert!(first_visual.frame.y > first.frame.y);
    assert_eq!(
        first_visual.component_role.as_str(),
        "asset-thumbnail-visual"
    );
    assert_eq!(first_visual.component_variant.as_str(), "asset-mesh");
    assert_eq!(
        first_band.surface_variant.as_str(),
        "asset-thumbnail-name-area"
    );
    assert!(first_band.selected);
    assert_eq!(first_band.corner_radius, 4.0);
    assert!(first_band.frame.y > first_visual.frame.y);
    assert!(first_visual.frame.y + first_visual.frame.height <= first_band.frame.y);
    assert!(
        first_visual.frame.height >= first_band.frame.height * 1.45,
        "thumbnail area should dominate the name area like UE AssetTileItem: visual={:?}, band={:?}",
        first_visual.frame,
        first_band.frame
    );
    assert_eq!(first_marker.surface_variant.as_str(), "accent");
    assert!(first_marker.frame.width <= 0.0);
    assert_eq!(first_marker.frame.height, first_band.frame.height);
    assert_eq!(first_marker.frame.x, first_band.frame.x);
    assert_eq!(first_marker.frame.y, first_band.frame.y);
    assert!(
        first_marker.frame.width < first_band.frame.width * 0.04,
        "selected thumbnail state should be carried by the full card outline, not a bright info-band strip: marker={:?}, band={:?}",
        first_marker.frame,
        first_band.frame
    );
    assert!(first_name.frame.x > first_band.frame.x);
    assert!(first_name.frame.y > first_band.frame.y);
    assert_eq!(first_name_continuation.frame.height, 0.0);
    assert!(
        first_band.frame.height <= 44.0,
        "single-line thumbnail info bands should stay compact so the preview canvas dominates: {:?}",
        first_band.frame
    );
    assert!(
        first_visual.frame.height >= 86.0,
        "single-line thumbnail tiles should return vertical space to the preview canvas: visual={:?}, band={:?}",
        first_visual.frame,
        first_band.frame
    );
    assert_eq!(
        first_type_badge.surface_variant.as_str(),
        "asset-type-badge"
    );
    assert_eq!(first_type.text.as_str(), "MSH");
    assert_eq!(first_type.font_size, 8.5);
    assert_eq!(first_meta.font_size, 8.5);
    assert!(first_type_badge.frame.y > first_name.frame.y);
    assert_eq!(first_type.frame.x, first_type_badge.frame.x + 5.0);
    assert!(
        first_type_badge.frame.width >= 40.0,
        "type badge should reserve enough width for a three-letter resource code: {:?}",
        first_type_badge.frame
    );
    assert!(
        first_type.frame.width >= 32.0,
        "type label should not be squeezed into ellipsis width: {:?}",
        first_type.frame
    );
    assert!(first_type.frame.width < first_type_badge.frame.width);
    assert!(first_meta.frame.x > first_type_badge.frame.x + first_type_badge.frame.width);
    assert_eq!(first_meta.text.as_str(), "Ready");
    assert!(first_meta.frame.y > first_name.frame.y);
}

#[test]
fn thumbnail_view_keeps_file_like_names_single_line_with_extension_tail() {
    let mut asset = asset_item(1, true);
    asset.display_name = "workbench_host_window.zui".to_string();
    asset.file_name = "workbench_host_window.zui".to_string();
    asset.extension = "zui".to_string();
    asset.kind = ResourceKind::UiLayout;
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset].into(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let band = find_node(&nodes, "AssetBrowserThumbInfoBand01");
    let name = find_node(&nodes, "AssetBrowserThumbName01");
    let continuation = find_node(&nodes, "AssetBrowserThumbNameContinuation01");
    let type_badge = find_node(&nodes, "AssetBrowserThumbTypeBadge01");
    let meta = find_node(&nodes, "AssetBrowserThumbMeta01");

    assert!(name.text.as_str().ends_with(".zui"));
    assert!(
        measure_runtime_text_width(name.text.as_str(), name.font_size) <= name.frame.width + 0.01,
        "thumbnail file-like name should fit its measured text slot: name={:?}",
        name
    );
    assert!(name.text.as_str().ends_with(".zui"));
    assert!(continuation.text.is_empty());
    assert_eq!(continuation.frame.height, 0.0);
    assert!(
        type_badge.frame.y > name.frame.y,
        "type badge should sit below the single file title: badge={:?}, name={:?}",
        type_badge.frame,
        name.frame
    );
    assert!(
        type_badge.frame.y - name.frame.y <= 24.0,
        "single-line file titles should keep a compact Content Browser row rhythm: badge={:?}, name={:?}",
        type_badge.frame,
        name.frame
    );
    assert_eq!(meta.frame.y, type_badge.frame.y);
    assert!(
        type_badge.frame.y + type_badge.frame.height <= band.frame.y + band.frame.height,
        "type/status row should stay inside info band: band={:?}, badge={:?}",
        band.frame,
        type_badge.frame
    );
}

#[test]
fn thumbnail_view_uses_slate_tile_name_area_typography_and_row_rhythm() {
    let mut asset = asset_item(1, true);
    asset.display_name = "NavigationSettingsRuntimeProfile".to_string();
    asset.file_name = "NavigationSettingsRuntimeProfile".to_string();
    asset.extension = String::new();
    asset.kind = ResourceKind::Data;
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset].into(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let band = find_node(&nodes, "AssetBrowserThumbInfoBand01");
    let name = find_node(&nodes, "AssetBrowserThumbName01");
    let continuation = find_node(&nodes, "AssetBrowserThumbNameContinuation01");
    let type_badge = find_node(&nodes, "AssetBrowserThumbTypeBadge01");
    let meta = find_node(&nodes, "AssetBrowserThumbMeta01");

    assert!(
        band.frame.height >= 50.0,
        "thumbnail name area should reserve a real two-line tile rhythm: band={:?}",
        band.frame
    );
    assert_eq!(name.font_size, 10.0);
    assert_eq!(name.font_weight, 500);
    assert_eq!(name.text.as_str(), "NavigationSettings");
    assert_eq!(continuation.text.as_str(), "RuntimeProfile");
    assert_eq!(continuation.font_size, 9.0);
    assert_eq!(continuation.font_weight, 400);
    assert_eq!(continuation.text_tone.as_str(), "muted");
    assert!(
        type_badge.frame.y >= continuation.frame.y + continuation.frame.height + 5.0,
        "meta row should breathe below the continuation line: badge={:?}, continuation={:?}",
        type_badge.frame,
        continuation.frame
    );
    assert_eq!(meta.frame.y, type_badge.frame.y);
    assert!(
        type_badge.frame.y + type_badge.frame.height <= band.frame.y + band.frame.height - 3.0,
        "meta row should keep bottom padding inside the name area: band={:?}, badge={:?}",
        band.frame,
        type_badge.frame
    );
}

#[test]
fn thumbnail_view_uses_short_readable_type_badges_for_dense_resource_tiles() {
    let resource_kinds = [
        (ResourceKind::UiLayout, "UIL"),
        (ResourceKind::UiStyle, "UIS"),
        (ResourceKind::Texture, "TEX"),
        (ResourceKind::UiWidget, "UIW"),
        (ResourceKind::Material, "MAT"),
        (ResourceKind::Scene, "SCN"),
        (ResourceKind::Shader, "SHD"),
        (ResourceKind::Prefab, "PFB"),
    ];
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: resource_kinds
            .iter()
            .enumerate()
            .map(|(index, (kind, _))| {
                let mut asset = asset_item(index + 1, index == 0);
                asset.kind = *kind;
                asset.asset_type =
                    crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                        *kind,
                    );
                asset
            })
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    for (index, (_, expected_code)) in resource_kinds.iter().enumerate() {
        let control_suffix = format!("{:02}", index + 1);
        let badge = find_node(
            &nodes,
            format!("AssetBrowserThumbTypeBadge{control_suffix}").as_str(),
        );
        let label = find_node(
            &nodes,
            format!("AssetBrowserThumbType{control_suffix}").as_str(),
        );
        assert_eq!(label.text.as_str(), *expected_code);
        assert!(
            label.text.chars().count() <= 3,
            "{} should use a compact badge code",
            label.control_id
        );
        assert!(
            badge.frame.width >= 40.0 && label.frame.width >= 32.0,
            "{} should reserve readable pill geometry: badge={:?}, label={:?}",
            label.control_id,
            badge.frame,
            label.frame
        );
    }
}

#[test]
fn thumbnail_view_wraps_cards_on_narrow_content_width() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=6).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(420.0, 360.0));
    let first = find_node(&nodes, "AssetBrowserThumbCard01");
    let third = find_node(&nodes, "AssetBrowserThumbCard03");
    let fourth = find_node(&nodes, "AssetBrowserThumbCard04");

    assert!(
        third.frame.x > first.frame.x,
        "third card should stay on the first narrow row: first={:?}, third={:?}",
        first.frame,
        third.frame
    );
    assert!(
        fourth.frame.y > first.frame.y,
        "fourth card should wrap onto the second row: first={:?}, fourth={:?}",
        first.frame,
        fourth.frame
    );
}

#[test]
fn narrow_asset_toolbar_keeps_direct_asset_actions_available() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=6).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(420.0, 360.0));
    let search = find_node(&nodes, "SearchEdited");
    let list = find_node(&nodes, "AssetBrowserViewModeListButton");
    let thumbnail = find_node(&nodes, "AssetBrowserViewModeThumbButton");
    let locate = find_node(&nodes, "LocateSelectedAsset");
    let import = find_node(&nodes, "ImportModel");

    for action in [&list, &thumbnail, &locate] {
        assert_eq!(action.frame.width, 30.0);
        assert_eq!(action.frame.height, 30.0);
        assert!(
            action.frame.x >= search.frame.x + search.frame.width,
            "{} must remain reachable after the narrow search field: action={:?}, search={:?}",
            action.control_id,
            action.frame,
            search.frame
        );
    }
    assert_eq!(thumbnail.frame.x, list.frame.x + list.frame.width + 4.0);
    assert!(
        locate.frame.x >= thumbnail.frame.x + thumbnail.frame.width,
        "Locate must remain available after the view-mode actions"
    );
    assert!(
        import.frame.x >= locate.frame.x + locate.frame.width,
        "Import must remain a distinct trailing action instead of overlapping direct asset actions"
    );
}

#[test]
fn thumbnail_view_keeps_selection_inside_tiles_without_inline_summary_card() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=8).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let content = find_node(&nodes, "AssetBrowserContentPanel");
    let grid = find_node(&nodes, "AssetBrowserThumbGridPanel");
    let selected_card = find_node(&nodes, "AssetBrowserThumbCard01");
    let selected_band = find_node(&nodes, "AssetBrowserThumbInfoBand01");
    let selected_marker = find_node(&nodes, "AssetBrowserThumbSelectionMarker01");
    let seventh = find_node(&nodes, "AssetBrowserThumbCard07");

    assert_control_absent(&nodes, "AssetBrowserContentPreviewCard");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewVisual");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewName");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewNameContinuation");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewMeta");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewTypeBadge");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewType");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewState");
    assert_control_absent(&nodes, "AssetBrowserContentPreviewRevision");
    assert!(selected_card.selected);
    assert_eq!(selected_card.border_width, 1.0);
    assert!(!selected_card.focused);
    assert!(selected_band.selected);
    assert_eq!(selected_marker.surface_variant.as_str(), "accent");
    assert!(
        grid.frame.height >= content.frame.height - 26.0,
        "thumbnail grid should own the recovered summary space: grid={:?}, content={:?}",
        grid.frame,
        content.frame
    );
    assert!(
        seventh.frame.y > selected_card.frame.y,
        "recovered space should let the thumbnail grid show the next row: selected={:?}, seventh={:?}",
        selected_card.frame,
        seventh.frame
    );
}

#[test]
fn thumbnail_view_keeps_two_line_selected_names_on_tile_without_inline_summary_card() {
    let mut asset = asset_item(1, true);
    asset.display_name = "NavigationSettingsRuntimeProfile".to_string();
    asset.file_name = "NavigationSettingsRuntimeProfile".to_string();
    asset.extension = String::new();
    asset.kind = ResourceKind::Data;
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset].into(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let name = find_node(&nodes, "AssetBrowserThumbName01");
    let continuation = find_node(&nodes, "AssetBrowserThumbNameContinuation01");
    let type_badge = find_node(&nodes, "AssetBrowserThumbTypeBadge01");
    let meta = find_node(&nodes, "AssetBrowserThumbMeta01");

    assert_eq!(name.text.as_str(), "NavigationSettings");
    assert_eq!(continuation.text.as_str(), "RuntimeProfile");
    assert_eq!(continuation.frame.x, name.frame.x);
    assert!(continuation.frame.y > name.frame.y);
    assert!(continuation.frame.height > 0.0);
    assert_control_absent(&nodes, "AssetBrowserContentPreviewCard");
    assert!(
        type_badge.frame.y >= continuation.frame.y + continuation.frame.height,
        "thumbnail meta row should sit below the second name line: badge={:?}, continuation={:?}",
        type_badge.frame,
        continuation.frame
    );
    assert_eq!(meta.frame.y, type_badge.frame.y);
}
