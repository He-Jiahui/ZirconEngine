use crate::ui::layouts::views::{asset_browser_pane_nodes, ViewTemplateNodeData};
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetSelectionSnapshot, AssetUtilityTab, AssetViewMode,
    AssetWorkspaceSnapshot,
};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSET_BROWSER_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/asset_browser.zui"
));
const FRAME_EPSILON: f32 = 0.001;

#[test]
fn asset_browser_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(ASSET_BROWSER_LAYOUT_TOML).expect("asset layout");

    for required_node in [
        "asset_browser_root",
        "toolbar_panel",
        "toolbar_title_row",
        "toolbar_title_text",
        "toolbar_locate_button",
        "toolbar_subtitle_row",
        "toolbar_subtitle_text",
        "toolbar_search_row",
        "toolbar_search_field",
        "toolbar_kind_primary_row",
        "toolbar_kind_all_chip",
        "toolbar_kind_texture_chip",
        "toolbar_kind_material_chip",
        "toolbar_kind_scene_chip",
        "toolbar_kind_model_chip",
        "toolbar_kind_shader_chip",
        "toolbar_view_mode_list_button",
        "toolbar_view_mode_thumb_button",
        "toolbar_kind_secondary_row",
        "toolbar_kind_physics_chip",
        "toolbar_kind_skeleton_chip",
        "toolbar_kind_clip_chip",
        "toolbar_kind_sequence_chip",
        "toolbar_kind_graph_chip",
        "toolbar_kind_state_chip",
        "import_panel",
        "import_label",
        "import_path_field",
        "import_button",
        "main_panel",
        "sources_panel",
        "sources_header_panel",
        "sources_title_text",
        "sources_subtitle_text",
        "sources_divider",
        "sources_scroll_body",
        "sources_row_panel",
        "sources_row_icon",
        "sources_row_name_text",
        "sources_row_count_text",
        "content_panel",
        "details_panel",
        "details_header_panel",
        "details_header_title_text",
        "details_header_selection_text",
        "details_divider",
        "details_scroll_body",
        "details_content_panel",
        "details_preview_panel",
        "details_preview_visual_panel",
        "details_preview_name_text",
        "details_preview_locator_text",
        "details_preview_kind_text",
        "details_preview_identity_text",
        "details_preview_adapter_text",
        "details_preview_meta_path_text",
        "details_preview_diagnostics_text",
        "details_locator_panel",
        "details_locator_label",
        "details_locator_value",
        "details_type_panel",
        "details_type_label",
        "details_type_value",
        "details_identity_panel",
        "details_identity_label",
        "details_identity_uuid_value",
        "details_identity_revision_value",
        "details_metadata_panel",
        "details_metadata_label",
        "details_metadata_meta_path_value",
        "details_metadata_adapter_value",
        "details_diagnostics_panel",
        "details_diagnostics_label",
        "details_diagnostics_text",
        "utility_panel",
        "utility_tabs_row",
        "utility_preview_button",
        "utility_references_button",
        "utility_metadata_button",
        "utility_plugins_button",
        "utility_selection_locator_text",
        "utility_divider",
        "utility_content_panel",
        "preview_panel",
        "metadata_meta_path_panel",
        "metadata_adapter_panel",
        "metadata_diagnostics_panel",
        "plugins_panel",
        "reference_left_panel",
        "reference_right_panel",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "asset browser bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn asset_browser_projection_maps_bootstrap_asset_into_mount_nodes() {
    let nodes = collect_projected_nodes(AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        utility_tab: AssetUtilityTab::Metadata,
        kind_filter: Some(ResourceKind::Material),
        search_query: "mat".to_string(),
        selected_folder_id: Some("res://materials".to_string()),
        ..AssetWorkspaceSnapshot::default()
    });

    assert!(
        !nodes.is_empty(),
        "asset browser projection should produce template mount nodes"
    );

    let toolbar = find_node(&nodes, "AssetBrowserToolbarPanel");
    assert_eq!(toolbar.role.to_string(), "Panel");
    assert_eq!(toolbar.surface_variant.to_string(), "frame_only");
    assert!(toolbar.frame.width > 0.0 && toolbar.frame.height > 0.0);

    let title = find_node(&nodes, "AssetBrowserTitleText");
    let locate = find_node(&nodes, "LocateSelectedAsset");
    let subtitle = find_node(&nodes, "AssetBrowserSubtitleText");
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.control_id == "SearchEdited")
            .count(),
        1,
        "TextField projection should not keep both component and generated text nodes"
    );
    let search = find_node(&nodes, "SearchEdited");
    let import_panel = find_node(&nodes, "AssetBrowserImportPanel");
    let import_label = find_node(&nodes, "AssetBrowserImportLabel");
    let import_path = find_node(&nodes, "AssetBrowserImportPathField");
    let import_button = find_node(&nodes, "ImportModel");
    let main = find_node(&nodes, "AssetBrowserMainPanel");
    let sources = find_node(&nodes, "AssetBrowserSourcesPanel");
    let sources_title = find_node(&nodes, "AssetBrowserSourcesTitleText");
    let sources_subtitle = find_node(&nodes, "AssetBrowserSourcesSubtitleText");
    let sources_scroll_body = find_node(&nodes, "AssetBrowserSourcesScrollBody");
    let content = find_node(&nodes, "AssetBrowserContentPanel");
    let table_header = find_node(&nodes, "WorkbenchAssetBrowserTableHeader");
    let table_row = find_node(&nodes, "WorkbenchAssetBrowserAssetRow01");
    let details = find_node(&nodes, "AssetBrowserDetailsPanel");
    let details_title = find_node(&nodes, "AssetBrowserDetailsHeaderTitleText");
    let details_header = find_node(&nodes, "AssetBrowserDetailsHeaderPanel");
    let details_scroll_body = find_node(&nodes, "AssetBrowserDetailsScrollBody");
    let details_content = find_node(&nodes, "AssetBrowserDetailsContentPanel");
    let details_preview = find_node(&nodes, "AssetBrowserDetailsPreviewPanel");
    let details_locator = find_node(&nodes, "AssetBrowserDetailsLocatorPanel");
    let details_diagnostics = find_node(&nodes, "AssetBrowserDetailsDiagnosticsPanel");
    let utility = find_node(&nodes, "AssetBrowserUtilityPanel");
    let utility_tabs = find_node(&nodes, "AssetBrowserUtilityTabsRow");
    let utility_selection = find_node(&nodes, "AssetBrowserSelectionLocatorText");
    let utility_divider = find_node(&nodes, "AssetBrowserUtilityDivider");
    let utility_content = find_node(&nodes, "AssetBrowserUtilityContentPanel");
    let meta_path_panel = find_node(&nodes, "AssetBrowserMetaPathPanel");
    let diagnostics_text = find_node(&nodes, "AssetBrowserDiagnosticsText");
    let list_mode = find_node(&nodes, "AssetBrowserViewModeListButton");
    let thumb_mode = find_node(&nodes, "AssetBrowserViewModeThumbButton");
    let preview_tab = find_node(&nodes, "AssetBrowserPreviewTabButton");
    let references_tab = find_node(&nodes, "AssetBrowserReferencesTabButton");
    let metadata_tab = find_node(&nodes, "AssetBrowserMetadataTabButton");
    let plugins_tab = find_node(&nodes, "AssetBrowserPluginsTabButton");
    let material_chip = find_node(&nodes, "AssetBrowserKindMaterialChip");

    assert_eq!(title.text.to_string(), "Asset Browser");
    assert_eq!(sources_title.text.to_string(), "Sources");
    assert_eq!(sources_subtitle.text.to_string(), "Project/assets");
    assert_eq!(sources_scroll_body.role.to_string(), "Panel");
    assert_eq!(
        sources_scroll_body.surface_variant.to_string(),
        "scroll-body"
    );
    assert_eq!(locate.role.to_string(), "Button");
    assert_eq!(locate.text.to_string(), "Locate In Assets");
    assert_eq!(locate.dispatch_kind.to_string(), "asset:browser");
    assert_eq!(
        locate.binding_id.to_string(),
        "AssetSurface/LocateSelectedAsset"
    );
    assert_eq!(subtitle.text.to_string(), "Project/assets");
    assert_eq!(search.role.to_string(), "InputField");
    assert_eq!(search.component_role.to_string(), "input-field");
    assert_eq!(search.text.to_string(), "mat");
    assert_eq!(search.value_text.to_string(), "mat");
    assert_eq!(search.dispatch_kind.to_string(), "asset:browser");
    assert_eq!(search.binding_id.to_string(), "AssetSurface/SearchEdited");
    assert_eq!(
        search.edit_action_id.to_string(),
        "AssetSurface/SearchEdited"
    );
    assert_eq!(search.commit_action_id.to_string(), "");
    assert_eq!(import_label.text.to_string(), "Quick Import");
    assert_eq!(import_path.role.to_string(), "InputField");
    assert_eq!(import_path.component_role.to_string(), "input-field");
    assert_eq!(
        import_path.text.to_string(),
        "Drop or paste asset source path"
    );
    assert_eq!(import_path.value_text.to_string(), "");
    assert_eq!(import_button.role.to_string(), "Button");
    assert_eq!(import_button.button_variant.to_string(), "primary");
    assert_eq!(import_button.dispatch_kind.to_string(), "asset:browser");
    assert_eq!(
        import_button.binding_id.to_string(),
        "AssetSurface/ImportModel"
    );
    assert_eq!(
        shared_string_model_values(&table_header.options),
        ["Name", "Type", "Size", "Rev"],
        "asset table headers should use declared cells instead of whitespace-split text"
    );
    assert_eq!(
        shared_string_model_values(&table_row.options),
        ["Empty Asset", "Asset", "0KB", "pending"],
        "asset table rows should carry four explicit cells for stable retained-host table painting"
    );
    assert_eq!(details_title.text.to_string(), "Details");
    assert!(!details_header.selected);
    assert_eq!(details_preview.role.to_string(), "Panel");
    assert_eq!(
        details_preview.surface_variant.to_string(),
        "asset-placeholder"
    );
    assert_eq!(details_preview.border_width, 0.0);
    assert_eq!(details_locator.role.to_string(), "Panel");
    assert_eq!(details_locator.surface_variant.to_string(), "inset");
    assert_eq!(details_diagnostics.role.to_string(), "Panel");
    assert_eq!(details_diagnostics.surface_variant.to_string(), "inset");
    assert_eq!(
        utility_selection.text.to_string(),
        "Select an asset to inspect"
    );
    assert_eq!(meta_path_panel.role.to_string(), "Panel");
    assert_eq!(meta_path_panel.surface_variant.to_string(), "inset");
    assert_eq!(diagnostics_text.text.to_string(), "No active diagnostics");
    assert_eq!(list_mode.text.to_string(), "List");
    assert_eq!(list_mode.value_text.to_string(), "list");
    assert!(thumb_mode.selected);
    assert_eq!(thumb_mode.text.to_string(), "Thumb");
    assert_eq!(thumb_mode.surface_variant.to_string(), "inset");
    assert_eq!(thumb_mode.dispatch_kind.to_string(), "asset:browser");
    assert_eq!(
        thumb_mode.action_id.to_string(),
        "workbench.asset.view_mode.set"
    );
    assert_eq!(
        thumb_mode.binding_id.to_string(),
        "AssetSurface/SetViewMode"
    );
    assert_eq!(thumb_mode.value_text.to_string(), "thumbnail");
    assert_eq!(preview_tab.text.to_string(), "Preview");
    assert_eq!(preview_tab.value_text.to_string(), "preview");
    assert_eq!(references_tab.text.to_string(), "References");
    assert_eq!(references_tab.value_text.to_string(), "references");
    assert!(metadata_tab.selected);
    assert_eq!(metadata_tab.text.to_string(), "Metadata");
    assert_eq!(metadata_tab.surface_variant.to_string(), "inset");
    assert_eq!(metadata_tab.dispatch_kind.to_string(), "asset:browser");
    assert_eq!(
        metadata_tab.action_id.to_string(),
        "workbench.asset.utility_tab.set"
    );
    assert_eq!(
        metadata_tab.binding_id.to_string(),
        "AssetSurface/SetUtilityTab"
    );
    assert_eq!(metadata_tab.value_text.to_string(), "metadata");
    assert_eq!(plugins_tab.text.to_string(), "Plugins");
    assert_eq!(plugins_tab.value_text.to_string(), "plugins");
    assert!(material_chip.selected);
    assert!(meta_path_panel.selected);
    assert_eq!(material_chip.surface_variant.to_string(), "inset");
    assert_eq!(material_chip.dispatch_kind.to_string(), "asset:browser");
    assert_eq!(
        material_chip.action_id.to_string(),
        "workbench.asset.kind_filter.set"
    );
    assert_eq!(
        material_chip.binding_id.to_string(),
        "AssetSurface/SetKindFilter"
    );
    assert_eq!(material_chip.value_text.to_string(), "Material");
    assert_frame_value(
        "toolbar import panel y",
        import_panel.frame.y,
        toolbar.frame.y,
    );
    assert_frame_value(
        "toolbar import panel height",
        import_panel.frame.height,
        toolbar.frame.height,
    );
    assert_frame_value(
        "toolbar main panel y",
        main.frame.y,
        toolbar.frame.y + toolbar.frame.height + 6.0,
    );
    assert!(sources.frame.x >= main.frame.x);
    assert!(content.frame.x >= sources.frame.x + sources.frame.width);
    assert!(details.frame.x >= content.frame.x + content.frame.width);
    assert!(details_title.frame.y >= details.frame.y);
    assert!(details_scroll_body.frame.y >= details.frame.y);
    assert!(details_content.frame.height > details_scroll_body.frame.height);
    assert!(details_preview.frame.width > 0.0 && details_preview.frame.height > 0.0);
    assert!(utility.frame.y >= main.frame.y + main.frame.height);
    assert!(utility_tabs.frame.y >= utility.frame.y);
    assert!(utility_selection.frame.width > 0.0);
    assert!(utility_divider.frame.height > 0.0);
    assert!(utility_content.frame.y >= utility_tabs.frame.y + utility_tabs.frame.height);
    assert_frame_value("utility panel height", utility.frame.height, 144.0);
    assert_frame_value("utility tabs height", utility_tabs.frame.height, 22.0);
    assert_frame_value(
        "utility content y offset",
        utility_content.frame.y - utility.frame.y,
        28.0,
    );
    assert_frame_value(
        "utility content height",
        utility_content.frame.height,
        116.0,
    );
    assert_frame_value(
        "utility divider y offset",
        utility_divider.frame.y - utility.frame.y,
        26.0,
    );
    assert_frame_value("preview tab width", preview_tab.frame.width, 68.0);
    assert_frame_value("references tab width", references_tab.frame.width, 92.0);
    assert_frame_value("metadata tab width", metadata_tab.frame.width, 84.0);
    assert_frame_value("plugins tab width", plugins_tab.frame.width, 72.0);
    assert_frame_value(
        "utility selection label width",
        utility_selection.frame.width,
        156.0,
    );
    assert!(import_path.frame.width > 0.0 && import_path.frame.height > 0.0);
    assert!(meta_path_panel.frame.width > 0.0 && meta_path_panel.frame.height > 0.0);
    assert_control_absent(&nodes, "AssetBrowserPreviewPanel");
    assert_control_absent(&nodes, "AssetBrowserReferenceLeftPanel");
    assert_control_absent(&nodes, "AssetBrowserReferenceRightPanel");
    assert_control_absent(&nodes, "AssetBrowserPluginsPanel");
}

#[test]
fn asset_browser_projection_keeps_only_preview_content_for_preview_tab() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        utility_tab: AssetUtilityTab::Preview,
        selected_asset_uuid: Some("asset-ui-layout".to_string()),
        visible_assets: vec![sample_asset(
            "asset-ui-layout",
            "res://ui/editor/workbench_page_chrome.zui",
            "workbench_page_chrome.zui",
            ResourceKind::UiLayout,
            true,
        )],
        ..AssetWorkspaceSnapshot::default()
    };
    let nodes = collect_projected_nodes(snapshot);

    let preview_panel = find_node(&nodes, "AssetBrowserPreviewPanel");
    let preview_visual = find_node(&nodes, "AssetBrowserPreviewVisualPanel");
    let preview_name = find_node(&nodes, "AssetBrowserPreviewNameText");
    let preview_locator = find_node(&nodes, "AssetBrowserPreviewLocatorText");
    let utility_selection = find_node(&nodes, "AssetBrowserSelectionLocatorText");
    let content_card = find_node(&nodes, "AssetBrowserContentPreviewCard");
    let details_preview = find_node(&nodes, "AssetBrowserDetailsPreviewPanel");

    assert!(preview_panel.selected);
    assert_eq!(preview_panel.surface_variant.to_string(), "asset-preview");
    assert_eq!(preview_panel.border_width, 1.0);
    assert_eq!(
        preview_visual.surface_variant.to_string(),
        "asset-preview-visual"
    );
    assert_eq!(preview_visual.border_width, 1.0);
    assert_eq!(preview_name.text.to_string(), "workbench_page_chrome.zui");
    assert_eq!(
        preview_locator.text.to_string(),
        "res://ui/editor/workbench_page_chrome.zui"
    );
    assert_eq!(utility_selection.text, preview_locator.text);
    assert!(content_card.selected);
    assert_eq!(content_card.surface_variant.to_string(), "asset-preview");
    assert!(!details_preview.selected);
    assert_eq!(
        details_preview.surface_variant.to_string(),
        "asset-placeholder"
    );
    assert_eq!(details_preview.border_width, 0.0);
    assert_control_absent(&nodes, "AssetBrowserMetaPathPanel");
    assert_control_absent(&nodes, "AssetBrowserReferenceLeftPanel");
    assert_control_absent(&nodes, "AssetBrowserReferenceRightPanel");
    assert_control_absent(&nodes, "AssetBrowserPluginsPanel");
}

#[test]
fn asset_browser_projection_compacts_preview_utility_for_short_viewport() {
    let nodes = collect_projected_nodes_with_size(
        AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::List,
            utility_tab: AssetUtilityTab::Preview,
            selected_asset_uuid: Some("asset-ui-layout".to_string()),
            visible_assets: vec![sample_asset(
                "asset-ui-layout",
                "res://ui/editor/workbench_page_chrome.zui",
                "workbench_page_chrome.zui",
                ResourceKind::UiLayout,
                true,
            )],
            selection: AssetSelectionSnapshot {
                uuid: Some("asset-ui-layout".to_string()),
                display_name: "workbench_page_chrome.zui".to_string(),
                locator: "res://ui/editor/workbench_page_chrome.zui".to_string(),
                kind: Some(ResourceKind::UiLayout),
                ..AssetSelectionSnapshot::default()
            },
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(828.0, 497.0),
    );

    let toolbar = find_node(&nodes, "AssetBrowserToolbarPanel");
    let toolbar_title_row = find_node(&nodes, "AssetBrowserToolbarTitleRow");
    let toolbar_title = find_node(&nodes, "AssetBrowserTitleText");
    let toolbar_subtitle_row = find_node(&nodes, "AssetBrowserToolbarSubtitleRow");
    let toolbar_subtitle = find_node(&nodes, "AssetBrowserSubtitleText");
    let search_row = find_node(&nodes, "AssetBrowserToolbarSearchRow");
    let search_field = find_node(&nodes, "SearchEdited");
    let kind_row = find_node(&nodes, "AssetBrowserToolbarKindPrimaryRow");
    let all_chip = find_node(&nodes, "AssetBrowserKindAllChip");
    let shader_chip = find_node(&nodes, "AssetBrowserKindShaderChip");
    let list_button = find_node(&nodes, "AssetBrowserViewModeListButton");
    let thumb_button = find_node(&nodes, "AssetBrowserViewModeThumbButton");
    let import_panel = find_node(&nodes, "AssetBrowserImportPanel");
    let import_label = find_node(&nodes, "AssetBrowserImportLabel");
    let import_path = find_node(&nodes, "AssetBrowserImportPathField");
    let import_button = find_node(&nodes, "ImportModel");
    let main = find_node(&nodes, "AssetBrowserMainPanel");
    let utility = find_node(&nodes, "AssetBrowserUtilityPanel");
    let utility_content = find_node(&nodes, "AssetBrowserUtilityContentPanel");
    let preview_panel = find_node(&nodes, "AssetBrowserPreviewPanel");
    let sources_panel = find_node(&nodes, "AssetBrowserSourcesPanel");
    let sources_row = find_node(&nodes, "AssetBrowserSourcesRowPanel");
    let content_panel = find_node(&nodes, "AssetBrowserContentPanel");
    let content_header = find_node(&nodes, "AssetBrowserContentHeaderRow");
    let content_header_title = find_node(&nodes, "AssetBrowserContentHeaderTitleText");
    let content_header_path = find_node(&nodes, "AssetBrowserContentHeaderPathText");
    let table_panel = find_node(&nodes, "AssetBrowserAssetTablePanel");
    let table_header = find_node(&nodes, "WorkbenchAssetBrowserTableHeader");
    let table_row04 = find_node(&nodes, "WorkbenchAssetBrowserAssetRow04");
    let content_preview_card = find_node(&nodes, "AssetBrowserContentPreviewCard");
    let content_preview_name = find_node(&nodes, "AssetBrowserContentPreviewName");
    let content_preview_name_continuation =
        find_node(&nodes, "AssetBrowserContentPreviewNameContinuation");
    let details_panel = find_node(&nodes, "AssetBrowserDetailsPanel");
    let details_preview_name = find_node(&nodes, "AssetBrowserDetailsPreviewNameText");

    assert_frame_value("compact toolbar height", toolbar.frame.height, 32.0);
    assert_eq!(toolbar_title_row.frame.height, 0.0);
    assert_eq!(toolbar_title.frame.height, 0.0);
    assert_eq!(toolbar_subtitle_row.frame.height, 0.0);
    assert_eq!(toolbar_subtitle.frame.height, 0.0);
    assert_frame_value("compact search row height", search_row.frame.height, 32.0);
    assert_frame_value("compact search row y", search_row.frame.y, toolbar.frame.y);
    assert_frame_value(
        "compact search field height",
        search_field.frame.height,
        30.0,
    );
    assert!(search_field.frame.width >= 160.0);
    assert_eq!(search_field.frame.y, toolbar.frame.y + 1.0);
    assert!(
        search_field.frame.x + search_field.frame.width < all_chip.frame.x,
        "compact search and kind filter should share one toolbar row without overlap"
    );
    assert_frame_value("compact kind row y", kind_row.frame.y, toolbar.frame.y);
    assert_frame_value("compact kind row height", kind_row.frame.height, 32.0);
    assert_eq!(all_chip.frame.y, toolbar.frame.y + 1.0);
    assert_eq!(shader_chip.frame.height, 0.0);
    assert_eq!(
        shader_chip.frame.width, 0.0,
        "short compact toolbar should keep only essential kind chips before view controls"
    );
    assert_frame_value("compact list button height", list_button.frame.height, 30.0);
    assert_frame_value(
        "compact thumb button height",
        thumb_button.frame.height,
        30.0,
    );
    assert!(thumb_button.frame.x > list_button.frame.x);
    assert_frame_value("compact import y", import_panel.frame.y, toolbar.frame.y);
    assert_frame_value("compact import height", import_panel.frame.height, 32.0);
    assert_frame_value(
        "compact import label height",
        import_label.frame.height,
        0.0,
    );
    assert_frame_value("compact import field width", import_path.frame.width, 0.0);
    assert_frame_value("compact import field height", import_path.frame.height, 0.0);
    assert_eq!(
        import_path.text.to_string(),
        "Drop or paste asset source path"
    );
    assert_eq!(import_path.value_text.to_string(), "");
    assert_frame_value(
        "compact import button height",
        import_button.frame.height,
        30.0,
    );
    assert!(
        thumb_button.frame.x + thumb_button.frame.width < import_button.frame.x,
        "compact import button should remain as the trailing command when the path field collapses"
    );
    assert_frame_value(
        "compact main panel y",
        main.frame.y,
        import_panel.frame.y + import_panel.frame.height + 6.0,
    );
    assert!(
        main.frame.y + main.frame.height + 6.0 <= utility.frame.y,
        "main panel should leave a stable gap before the compact utility drawer"
    );
    assert!(
        utility.frame.y + utility.frame.height <= 497.0,
        "compact utility drawer must stay inside the viewport"
    );
    assert_eq!(utility.frame.height, 28.0);
    assert_eq!(utility_content.frame.y, utility.frame.y + 28.0);
    assert_eq!(utility_content.frame.height, 0.0);
    assert_eq!(preview_panel.frame.height, 0.0);
    assert_all_nodes_collapsed(
        &nodes,
        &[
            "AssetBrowserPreviewPanel",
            "AssetBrowserPreviewVisualPanel",
            "AssetBrowserPreviewNameText",
            "AssetBrowserPreviewLocatorText",
            "AssetBrowserPreviewKindText",
            "AssetBrowserPreviewIdentityText",
            "AssetBrowserPreviewAdapterText",
            "AssetBrowserPreviewMetaPathText",
            "AssetBrowserPreviewDiagnosticsText",
        ],
    );
    assert_eq!(sources_panel.frame.width, 0.0);
    assert_eq!(sources_panel.frame.height, 0.0);
    assert_eq!(sources_row.frame.width, 0.0);
    assert_eq!(details_panel.frame.width, 0.0);
    assert_eq!(details_panel.frame.height, 0.0);
    assert!(content_preview_card.selected);
    assert_eq!(content_preview_name.text.to_string(), "workbench_page");
    assert_eq!(
        content_preview_name_continuation.text.to_string(),
        "chrome.zui"
    );
    assert_eq!(
        content_preview_name_continuation.frame.x,
        content_preview_name.frame.x
    );
    assert!(content_preview_name_continuation.frame.y > content_preview_name.frame.y);
    assert!(
        content_panel.frame.x <= main.frame.x + 1.0 && content_panel.frame.width > 740.0,
        "short compact viewport should give the primary asset list the sources and details columns"
    );
    assert_frame_value(
        "compact content header height",
        content_header.frame.height,
        20.0,
    );
    assert_frame_value(
        "compact table y offset",
        table_header.frame.y - content_header.frame.y,
        24.0,
    );
    assert!(content_header_title.frame.width > 0.0);
    assert!(content_header_path.frame.width >= 96.0);
    assert!(
        content_header_title.frame.x >= content_header.frame.x + 8.0
            && content_header_path.frame.x + content_header_path.frame.width
                <= content_header.frame.x + content_header.frame.width - 8.0,
        "compact content header text should stay padded inside the content surface"
    );
    assert_frame_value(
        "compact table panel closes on last visible row",
        table_panel.frame.y + table_panel.frame.height
            - (table_row04.frame.y + table_row04.frame.height),
        0.0,
    );
    assert_eq!(
        visible_node_count(&nodes, "AssetBrowserContentPanel"),
        1,
        "compact content panel should not leave a second visible projected container"
    );
    assert_eq!(
        visible_node_count(&nodes, "AssetBrowserAssetTablePanel"),
        1,
        "compact table panel should not leave a second visible projected container"
    );
    assert!(table_row04.frame.height <= 30.0);
    assert!(content_preview_card.frame.y >= table_row04.frame.y + table_row04.frame.height + 8.0);
    assert!(
        content_preview_card.frame.y + content_preview_card.frame.height
            <= content_panel.frame.y + content_panel.frame.height + FRAME_EPSILON
    );
    assert_eq!(details_preview_name.frame.width, 0.0);
}

fn assert_frame_value(label: &str, actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < FRAME_EPSILON,
        "{label} drifted: expected {expected}, got {actual}"
    );
}

fn visible_node_count(nodes: &[ViewTemplateNodeData], control_id: &str) -> usize {
    nodes
        .iter()
        .filter(|node| {
            node.control_id == control_id
                && node.frame.width > FRAME_EPSILON
                && node.frame.height > FRAME_EPSILON
        })
        .count()
}

fn assert_all_nodes_collapsed(nodes: &[ViewTemplateNodeData], control_ids: &[&str]) {
    for control_id in control_ids {
        let visible = visible_node_count(nodes, control_id);
        assert_eq!(
            visible, 0,
            "all `{control_id}` projections should collapse in compact short viewport"
        );
    }
}

fn collect_projected_nodes(snapshot: AssetWorkspaceSnapshot) -> Vec<ViewTemplateNodeData> {
    collect_projected_nodes_with_size(snapshot, UiSize::new(1280.0, 820.0))
}

fn collect_projected_nodes_with_size(
    snapshot: AssetWorkspaceSnapshot,
    size: UiSize,
) -> Vec<ViewTemplateNodeData> {
    let pane = asset_browser_pane_nodes(&snapshot, size);
    (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect()
}

fn find_node<'a>(nodes: &'a [ViewTemplateNodeData], control_id: &str) -> &'a ViewTemplateNodeData {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .unwrap_or_else(|| panic!("expected projected node `{control_id}`"))
}

fn assert_control_absent(nodes: &[ViewTemplateNodeData], control_id: &str) {
    assert!(
        nodes.iter().all(|node| node.control_id != control_id),
        "projected utility tab should not include inactive `{control_id}`"
    );
}

fn sample_asset(
    uuid: &str,
    locator: &str,
    display_name: &str,
    kind: ResourceKind,
    selected: bool,
) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        display_name: display_name.to_string(),
        file_name: display_name.to_string(),
        extension: String::new(),
        kind,
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(42),
    }
}

fn shared_string_model_values(
    model: &crate::ui::retained_host::primitives::ModelRc<
        crate::ui::retained_host::primitives::SharedString,
    >,
) -> Vec<String> {
    (0..model.row_count())
        .filter_map(|row| model.row_data(row))
        .map(|value| value.to_string())
        .collect()
}
