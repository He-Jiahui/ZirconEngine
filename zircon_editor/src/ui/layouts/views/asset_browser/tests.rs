use super::*;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

mod reference_lists;

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
        visible_assets: vec![asset],
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
        visible_assets: vec![asset],
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
        visible_assets: vec![asset],
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
        visible_assets: vec![asset],
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

fn asset_item(index: usize, selected: bool) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: format!("asset-{index:02}"),
        locator: format!("res://asset-{index:02}"),
        display_name: format!("Asset_{index:02}.mesh"),
        file_name: format!("Asset_{index:02}.mesh"),
        extension: "mesh".to_string(),
        kind: ResourceKind::Mesh,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::Mesh,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(index as u64),
    }
}

fn asset_folder(
    folder_id: &str,
    display_name: &str,
    depth: usize,
    selected: bool,
) -> AssetFolderSnapshot {
    AssetFolderSnapshot {
        folder_id: folder_id.to_string(),
        parent_folder_id: None,
        display_name: display_name.to_string(),
        recursive_asset_count: 0,
        depth,
        selected,
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
