use super::*;

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
    let filter_group = find_node(&nodes, "AssetBrowserToolbarKindPrimaryRow");
    let kind_all = find_node(&nodes, "AssetBrowserKindAllChip");
    let kind_texture = find_node(&nodes, "AssetBrowserKindTextureChip");
    let kind_material = find_node(&nodes, "AssetBrowserKindMaterialChip");
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
    for node in [
        &search,
        &filter_group,
        &kind_all,
        &kind_texture,
        &kind_material,
        &thumb,
        &import_button,
    ] {
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
        kind_texture.frame.width > 0.0 && kind_material.frame.width > 0.0,
        "900px toolbar should prioritize visible filters before the optional import path"
    );
    assert!(
        import_button.frame.x > thumb.frame.x + thumb.frame.width,
        "Import should remain a trailing command after the filter/view group"
    );
    assert_eq!(import_path.frame.width, 0.0);
    assert_eq!(import_path.frame.height, 0.0);
    assert_eq!(filter_group.surface_variant.as_str(), "inset");
    assert_eq!(filter_group.border_width, 1.0);
    assert_eq!(filter_group.corner_radius, 4.0);
    assert_eq!(filter_group.z_index, -1);
    assert!(filter_group.frame.x <= kind_all.frame.x);
    assert!(filter_group.frame.x + filter_group.frame.width >= thumb.frame.x + thumb.frame.width);
    assert!(
        filter_group.frame.width < toolbar.frame.width * 0.58,
        "filter group should wrap the compound controls instead of spanning the full toolbar: group={:?}, toolbar={:?}",
        filter_group.frame,
        toolbar.frame
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
fn asset_browser_utility_tabs_use_compact_slate_tab_strip_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=6).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let row = find_node(&nodes, "AssetBrowserUtilityTabsRow");
    let preview = find_node(&nodes, "AssetBrowserPreviewTabButton");
    let references = find_node(&nodes, "AssetBrowserReferencesTabButton");
    let metadata = find_node(&nodes, "AssetBrowserMetadataTabButton");
    let plugins = find_node(&nodes, "AssetBrowserPluginsTabButton");
    let locator = find_node(&nodes, "AssetBrowserSelectionLocatorText");
    let divider = find_node(&nodes, "AssetBrowserUtilityDivider");
    let content = find_node(&nodes, "AssetBrowserUtilityContentPanel");

    assert_eq!(row.frame.height, 22.0);
    assert_eq!(preview.frame.width, 68.0);
    assert_eq!(references.frame.width, 92.0);
    assert_eq!(metadata.frame.width, 84.0);
    assert_eq!(plugins.frame.width, 72.0);
    for tab in [&preview, &references, &metadata, &plugins] {
        assert_eq!(
            tab.frame.height, row.frame.height,
            "{} should fill the compact utility tab row height",
            tab.control_id
        );
        assert_eq!(
            tab.frame.y, row.frame.y,
            "{} should share the utility tab row baseline",
            tab.control_id
        );
        assert_eq!(
            tab.font_size, 12.0,
            "{} should opt into the readable UI tab font instead of falling back to the dense body text size",
            tab.control_id
        );
    }
    assert_eq!(
        preview.font_weight, 600,
        "the active utility tab should request the strong UI text face"
    );
    for tab in [&references, &metadata, &plugins] {
        assert_eq!(
            tab.font_weight, 400,
            "{} should keep the idle UI text weight",
            tab.control_id
        );
    }
    assert_eq!(
        references.frame.x - (preview.frame.x + preview.frame.width),
        6.0
    );
    assert_eq!(
        metadata.frame.x - (references.frame.x + references.frame.width),
        6.0
    );
    assert_eq!(
        plugins.frame.x - (metadata.frame.x + metadata.frame.width),
        6.0
    );
    assert_eq!(locator.frame.width, 156.0);
    assert_eq!(locator.frame.height, row.frame.height);
    assert!(locator.frame.x > plugins.frame.x + plugins.frame.width);
    assert_eq!(
        locator.frame.x + locator.frame.width,
        row.frame.x + row.frame.width
    );
    assert_eq!(divider.frame.y, row.frame.y + 26.0);
    assert_eq!(content.frame.y, row.frame.y + 28.0);
}

#[test]
fn compact_asset_browser_keeps_scaled_navigation_content_and_details_regions() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        selected_asset_uuid: Some("asset-01".to_string()),
        selected_folder_id: Some("materials".to_string()),
        folder_tree: vec![
            asset_folder("content", "Content", 0, false),
            asset_folder("materials", "Materials", 1, false),
        ],
        visible_assets: (1..=8).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let sources = find_node(&nodes, "AssetBrowserSourcesPanel");
    let source_row = find_node(&nodes, "AssetBrowserSourcesRowPanel");
    let selected_source_row = find_node(
        &nodes,
        "AssetBrowserSourcesTreeRow02/AssetBrowserSourcesRowPanel",
    );
    let content = find_node(&nodes, "AssetBrowserContentPanel");
    let details = find_node(&nodes, "AssetBrowserDetailsPanel");

    assert_eq!(sources.frame.width, 152.0);
    assert_eq!(details.frame.width, 204.0);
    assert!(
        content.frame.width >= 520.0,
        "the browser content region should remain the dominant width: {content:?}"
    );
    assert!(
        sources.frame.x + sources.frame.width < content.frame.x
            && content.frame.x + content.frame.width < details.frame.x,
        "all compact browser regions should remain ordered without overlap: sources={:?}, content={:?}, details={:?}",
        sources.frame,
        content.frame,
        details.frame
    );
    assert!(
        source_row.frame.x >= sources.frame.x
            && source_row.frame.x + source_row.frame.width <= sources.frame.x + sources.frame.width,
        "source tree row must be constrained by its compact navigation region: row={:?}, sources={:?}",
        source_row.frame,
        sources.frame
    );
    assert_eq!(source_row.role, "TreeRow");
    assert_eq!(source_row.text, "Content");
    assert_eq!(selected_source_row.text, "Materials");
    assert!(selected_source_row.selected);
    assert_eq!(selected_source_row.value_number, 1.0);
    assert_eq!(
        selected_source_row.frame.y - source_row.frame.y,
        32.0,
        "tree rows should retain the pointer bridge's 28px row plus 4px gap rhythm"
    );
}

#[test]
fn narrow_asset_browser_width_uses_compact_columns_even_when_height_is_available() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        visible_assets: (1..=8).map(|index| asset_item(index, false)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 800.0));
    let sources = find_node(&nodes, "AssetBrowserSourcesPanel");
    let content = find_node(&nodes, "AssetBrowserContentPanel");
    let details = find_node(&nodes, "AssetBrowserDetailsPanel");

    assert_eq!(sources.frame.width, 152.0);
    assert_eq!(details.frame.width, 204.0);
    assert!(
        content.frame.width >= 520.0,
        "narrow windows should keep content usable regardless of height: {content:?}"
    );
}

#[test]
fn asset_browser_projected_selection_does_not_impersonate_keyboard_focus() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        kind_filter: Some(ResourceKind::Texture),
        utility_tab: AssetUtilityTab::Metadata,
        selected_asset_uuid: Some("asset-01".to_string()),
        visible_assets: (1..=6).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));

    for control_id in [
        "AssetBrowserViewModeThumbButton",
        "AssetBrowserKindTextureChip",
        "AssetBrowserMetadataTabButton",
        "AssetBrowserMetaPathPanel",
        "AssetBrowserToolkitPanel",
        "AssetBrowserDiagnosticsPanel",
    ] {
        let node = find_node(&nodes, control_id);
        assert!(
            node.selected,
            "{control_id} should keep selected/active visual state"
        );
        assert!(
            !node.focused,
            "{control_id} should not synthesize keyboard focus from snapshot selection"
        );
    }

    for control_id in [
        "AssetBrowserViewModeListButton",
        "AssetBrowserKindMaterialChip",
        "AssetBrowserPreviewTabButton",
        "AssetBrowserReferencesTabButton",
        "AssetBrowserPluginsTabButton",
    ] {
        let node = find_node(&nodes, control_id);
        assert!(
            !node.selected,
            "{control_id} should remain idle for the texture/metadata snapshot"
        );
        assert!(
            !node.focused,
            "{control_id} should not carry stale focus while idle"
        );
    }
}

#[test]
fn asset_browser_utility_tab_projection_does_not_request_inset_surface() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        kind_filter: Some(ResourceKind::Texture),
        utility_tab: AssetUtilityTab::Metadata,
        visible_assets: (1..=4).map(|index| asset_item(index, index == 1)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let metadata = find_node(&nodes, "AssetBrowserMetadataTabButton");
    let preview = find_node(&nodes, "AssetBrowserPreviewTabButton");
    let texture = find_node(&nodes, "AssetBrowserKindTextureChip");

    assert!(metadata.selected);
    assert_eq!(metadata.surface_variant.as_str(), "");
    assert_eq!(metadata.text_tone.as_str(), "default");
    assert!(!preview.selected);
    assert_eq!(preview.surface_variant.as_str(), "");
    assert_eq!(preview.text_tone.as_str(), "subtle");
    assert!(texture.selected);
    assert_eq!(texture.surface_variant.as_str(), "inset");
}
