use super::*;
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn asset_browser_toolbar_uses_single_row_slate_compound_control_rhythm() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=6).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let toolbar = find_node(&nodes, "AssetBrowserToolbarPanel");
    let search = find_node(&nodes, "SearchEdited");
    let kind_all = find_node(&nodes, "AssetBrowserKindAllChip");
    let thumb = find_node(&nodes, "AssetBrowserViewModeThumbButton");
    let import_panel = find_node(&nodes, "AssetBrowserImportPanel");
    let import_label = find_node(&nodes, "AssetBrowserImportLabel");
    let import_path = find_node(&nodes, "AssetBrowserImportPathField");
    let import_button = find_node(&nodes, "ImportModel");
    let content = find_node(&nodes, "AssetBrowserContentPanel");

    assert!(
        toolbar.frame.height <= 34.0,
        "asset toolbar should collapse to a single Slate-like compound row: {:?}",
        toolbar.frame
    );
    for node in [&search, &kind_all, &thumb, &import_path, &import_button] {
        assert_eq!(
            node.frame.height, 30.0,
            "{} should share the toolbar control height",
            node.control_id
        );
        assert_eq!(
            node.frame.y,
            toolbar.frame.y + 1.0,
            "{} should align to the single toolbar row",
            node.control_id
        );
    }
    assert!(
        search.frame.width >= toolbar.frame.width * 0.32,
        "search should be the dominant flexible field: search={:?}, toolbar={:?}",
        search.frame,
        toolbar.frame
    );
    assert!(
        kind_all.frame.x > search.frame.x + search.frame.width,
        "kind chips should follow the search field on the same row"
    );
    assert!(
        import_path.frame.x > thumb.frame.x + thumb.frame.width,
        "Quick Import should be a trailing input group instead of a second toolbar row"
    );
    assert_eq!(import_label.frame.width, 0.0);
    assert_eq!(import_label.frame.height, 0.0);
    assert_eq!(import_panel.frame.y, toolbar.frame.y);
    assert_eq!(import_panel.frame.height, toolbar.frame.height);
    assert!(
        content.frame.y <= toolbar.frame.y + toolbar.frame.height + 8.0,
        "content should reclaim the old Quick Import row height: content={:?}, toolbar={:?}",
        content.frame,
        toolbar.frame
    );
}

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
    assert_eq!(first.border_width, 0.0);
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
    assert_eq!(first_marker.frame.height, 2.0);
    assert_eq!(first_marker.frame.y, first_band.frame.y);
    assert!(first_name.frame.x > first_band.frame.x);
    assert!(first_name.frame.y > first_band.frame.y);
    assert_eq!(first_name_continuation.frame.height, 0.0);
    assert_eq!(
        first_type_badge.surface_variant.as_str(),
        "asset-type-badge"
    );
    assert_eq!(first_type.text.as_str(), "MESH");
    assert!(first_type_badge.frame.y > first_name.frame.y);
    assert_eq!(first_type.frame.x, first_type_badge.frame.x + 4.0);
    assert!(first_type.frame.width < first_type_badge.frame.width);
    assert!(first_meta.frame.x > first_type_badge.frame.x + first_type_badge.frame.width);
    assert_eq!(first_meta.text.as_str(), "Ready");
    assert!(first_meta.frame.y > first_name.frame.y);
}

#[test]
fn thumbnail_view_places_two_line_names_above_type_status_row() {
    let mut asset = asset_item(1, true);
    asset.display_name = "workbench_host_window.zui".to_string();
    asset.file_name = "workbench_host_window.zui".to_string();
    asset.extension = "zui".to_string();
    asset.kind = ResourceKind::UiLayout;
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset],
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let band = find_node(&nodes, "AssetBrowserThumbInfoBand01");
    let name = find_node(&nodes, "AssetBrowserThumbName01");
    let continuation = find_node(&nodes, "AssetBrowserThumbNameContinuation01");
    let type_badge = find_node(&nodes, "AssetBrowserThumbTypeBadge01");
    let meta = find_node(&nodes, "AssetBrowserThumbMeta01");

    assert_eq!(name.text.as_str(), "workbench_host");
    assert_eq!(continuation.text.as_str(), "window.zui");
    assert!(continuation.frame.y > name.frame.y);
    assert!(continuation.frame.height > 0.0);
    assert!(
        type_badge.frame.y >= continuation.frame.y + continuation.frame.height,
        "type badge should sit below the second name line: badge={:?}, continuation={:?}",
        type_badge.frame,
        continuation.frame
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
    asset.display_name = "workbench_host_window.zui".to_string();
    asset.file_name = "workbench_host_window.zui".to_string();
    asset.extension = "zui".to_string();
    asset.kind = ResourceKind::UiLayout;
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset],
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
    assert_eq!(name.font_size, 9.0);
    assert_eq!(name.font_weight, 500);
    assert_eq!(continuation.font_size, 8.0);
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
    asset.display_name = "workbench_host_window.zui".to_string();
    asset.file_name = "workbench_host_window.zui".to_string();
    asset.extension = "zui".to_string();
    asset.kind = ResourceKind::UiLayout;
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: vec![asset],
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let name = find_node(&nodes, "AssetBrowserThumbName01");
    let continuation = find_node(&nodes, "AssetBrowserThumbNameContinuation01");
    let type_badge = find_node(&nodes, "AssetBrowserThumbTypeBadge01");
    let meta = find_node(&nodes, "AssetBrowserThumbMeta01");

    assert_eq!(name.text.as_str(), "workbench_host");
    assert_eq!(continuation.text.as_str(), "window.zui");
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

fn asset_item(index: usize, selected: bool) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: format!("asset-{index:02}"),
        locator: format!("res://asset-{index:02}"),
        display_name: format!("Asset_{index:02}.mesh"),
        file_name: format!("Asset_{index:02}.mesh"),
        extension: "mesh".to_string(),
        kind: ResourceKind::Mesh,
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(index as u64),
    }
}

fn find_node(
    nodes: &crate::ui::retained_host::primitives::ModelRc<ViewTemplateNodeData>,
    control_id: &str,
) -> ViewTemplateNodeData {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        if node.control_id.as_str() == control_id {
            return node;
        }
    }
    panic!("missing node {control_id}");
}

fn assert_control_absent(
    nodes: &crate::ui::retained_host::primitives::ModelRc<ViewTemplateNodeData>,
    control_id: &str,
) {
    for index in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(index) else {
            continue;
        };
        assert_ne!(
            node.control_id.as_str(),
            control_id,
            "thumbnail mode should not project `{control_id}`"
        );
    }
}
