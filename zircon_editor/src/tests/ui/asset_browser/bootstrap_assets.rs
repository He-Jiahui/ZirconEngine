use crate::ui::layouts::views::asset_browser_pane_nodes;
use slint::Model;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSET_BROWSER_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/asset_browser.ui.toml"
));

#[test]
fn asset_browser_bootstrap_layout_self_hosts_shell_sections() {
    let layout =
        crate::tests::support::load_test_ui_asset(ASSET_BROWSER_LAYOUT_TOML).expect("asset layout");

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
            layout.contains_node(required_node),
            "asset browser bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn asset_browser_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = asset_browser_pane_nodes(UiSize::new(1280.0, 820.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    assert!(
        !nodes.is_empty(),
        "asset browser projection should produce template mount nodes"
    );

    let toolbar = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserToolbarPanel")
        .expect("toolbar panel node");
    assert_eq!(toolbar.role.to_string(), "Mount");
    assert!(toolbar.frame.width > 0.0 && toolbar.frame.height > 0.0);

    let title = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserTitleText")
        .expect("title node");
    let locate = nodes
        .iter()
        .find(|node| node.control_id == "LocateSelectedAsset")
        .expect("locate button node");
    let subtitle = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSubtitleText")
        .expect("subtitle node");
    let search = nodes
        .iter()
        .find(|node| node.control_id == "SearchEdited")
        .expect("search node");
    let import_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserImportPanel")
        .expect("import panel node");
    let import_label = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserImportLabel")
        .expect("import label node");
    let import_path = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserImportPathField")
        .expect("import path node");
    let import_button = nodes
        .iter()
        .find(|node| node.control_id == "ImportModel")
        .expect("import button node");
    let main = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserMainPanel")
        .expect("main panel node");
    let sources = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSourcesPanel")
        .expect("sources panel node");
    let sources_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSourcesTitleText")
        .expect("sources title node");
    let sources_subtitle = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSourcesSubtitleText")
        .expect("sources subtitle node");
    let sources_scroll_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSourcesScrollBody")
        .expect("sources scroll body node");
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPanel")
        .expect("content panel node");
    let details = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsPanel")
        .expect("details panel node");
    let details_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsHeaderTitleText")
        .expect("details title node");
    let details_scroll_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsScrollBody")
        .expect("details scroll body node");
    let details_content = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsContentPanel")
        .expect("details content node");
    let details_preview = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsPreviewPanel")
        .expect("details preview node");
    let details_locator = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsLocatorPanel")
        .expect("details locator node");
    let details_diagnostics = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsDiagnosticsPanel")
        .expect("details diagnostics node");
    let utility = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityPanel")
        .expect("utility panel node");
    let utility_tabs = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityTabsRow")
        .expect("utility tabs node");
    let utility_selection = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSelectionLocatorText")
        .expect("utility selection node");
    let utility_divider = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityDivider")
        .expect("utility divider node");
    let utility_content = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityContentPanel")
        .expect("utility content node");
    let preview_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserPreviewPanel")
        .expect("preview panel node");
    let preview_visual = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserPreviewVisualPanel")
        .expect("preview visual node");
    let preview_name = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserPreviewNameText")
        .expect("preview name node");
    let reference_left = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceLeftPanel")
        .expect("left references node");
    let reference_left_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceLeftTitleText")
        .expect("left references title node");
    let reference_left_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceLeftScrollBody")
        .expect("left references body node");
    let reference_right = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceRightPanel")
        .expect("right references node");
    let reference_right_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceRightTitleText")
        .expect("right references title node");
    let meta_path_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserMetaPathPanel")
        .expect("meta path panel node");
    let diagnostics_text = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDiagnosticsText")
        .expect("diagnostics text node");
    let plugins_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserPluginsPanel")
        .expect("plugins panel node");

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
    assert_eq!(subtitle.text.to_string(), "Project/assets");
    assert_eq!(search.role.to_string(), "Mount");
    assert_eq!(import_label.text.to_string(), "Quick Import");
    assert_eq!(import_button.role.to_string(), "Button");
    assert_eq!(import_button.button_variant.to_string(), "primary");
    assert_eq!(details_title.text.to_string(), "Details");
    assert_eq!(details_preview.role.to_string(), "Panel");
    assert_eq!(details_preview.surface_variant.to_string(), "asset-preview");
    assert_eq!(details_locator.role.to_string(), "Panel");
    assert_eq!(details_locator.surface_variant.to_string(), "inset");
    assert_eq!(details_diagnostics.role.to_string(), "Panel");
    assert_eq!(details_diagnostics.surface_variant.to_string(), "danger");
    assert_eq!(
        utility_selection.text.to_string(),
        "Select an asset to inspect"
    );
    assert_eq!(preview_panel.role.to_string(), "Panel");
    assert_eq!(preview_panel.surface_variant.to_string(), "asset-preview");
    assert_eq!(preview_visual.role.to_string(), "Panel");
    assert_eq!(
        preview_visual.surface_variant.to_string(),
        "asset-preview-visual"
    );
    assert_eq!(preview_name.text.to_string(), "No Asset Selected");
    assert_eq!(reference_left_title.text.to_string(), "References");
    assert_eq!(reference_left_body.role.to_string(), "Panel");
    assert_eq!(
        reference_left_body.surface_variant.to_string(),
        "scroll-body"
    );
    assert_eq!(reference_right_title.text.to_string(), "Used By");
    assert_eq!(meta_path_panel.role.to_string(), "Panel");
    assert_eq!(meta_path_panel.surface_variant.to_string(), "inset");
    assert_eq!(diagnostics_text.text.to_string(), "No active diagnostics");
    assert_eq!(plugins_panel.role.to_string(), "Panel");
    assert_eq!(plugins_panel.surface_variant.to_string(), "inset");
    assert!(import_panel.frame.y >= toolbar.frame.y + toolbar.frame.height);
    assert!(main.frame.y >= import_panel.frame.y + import_panel.frame.height);
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
    assert!(import_path.frame.width > 0.0 && import_path.frame.height > 0.0);
    assert!(preview_panel.frame.width > 0.0 && preview_panel.frame.height > 0.0);
    assert!(reference_left.frame.width > 0.0 && reference_left.frame.height > 0.0);
    assert!(reference_right.frame.x >= reference_left.frame.x + reference_left.frame.width);
    assert!(meta_path_panel.frame.width > 0.0 && meta_path_panel.frame.height > 0.0);
    assert!(plugins_panel.frame.width > 0.0 && plugins_panel.frame.height > 0.0);
}
