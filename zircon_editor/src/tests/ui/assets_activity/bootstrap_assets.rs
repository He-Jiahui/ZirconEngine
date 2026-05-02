use crate::ui::layouts::views::assets_activity_pane_data;
use slint::Model;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/assets_activity.ui.toml"
));

#[test]
fn assets_activity_bootstrap_layout_self_hosts_shell_sections() {
    let layout = crate::tests::support::load_test_ui_asset(ASSETS_ACTIVITY_LAYOUT_TOML)
        .expect("assets activity layout");

    for required_node in [
        "assets_activity_root",
        "toolbar_panel",
        "toolbar_title_row",
        "toolbar_title_text",
        "toolbar_open_browser_button",
        "toolbar_subtitle_row",
        "toolbar_subtitle_text",
        "toolbar_search_row",
        "toolbar_search_field",
        "toolbar_view_mode_list_button",
        "toolbar_view_mode_thumb_button",
        "toolbar_kind_primary_row",
        "toolbar_kind_all_chip",
        "toolbar_kind_texture_chip",
        "toolbar_kind_material_chip",
        "toolbar_kind_scene_chip",
        "toolbar_kind_model_chip",
        "toolbar_kind_shader_chip",
        "toolbar_kind_secondary_row",
        "toolbar_kind_physics_chip",
        "toolbar_kind_skeleton_chip",
        "toolbar_kind_clip_chip",
        "toolbar_kind_sequence_chip",
        "toolbar_kind_graph_chip",
        "toolbar_kind_state_chip",
        "main_panel",
        "tree_panel",
        "tree_header_panel",
        "tree_title_text",
        "tree_subtitle_text",
        "tree_divider",
        "tree_scroll_body",
        "tree_row_panel",
        "tree_row_icon",
        "tree_row_name_text",
        "tree_row_count_text",
        "content_panel",
        "utility_panel",
        "utility_tabs_row",
        "utility_preview_button",
        "utility_references_button",
        "utility_selection_text",
        "utility_tabs_divider",
        "utility_content_panel",
        "preview_panel",
        "reference_left_panel",
        "reference_right_panel",
    ] {
        assert!(
            layout.contains_node(required_node),
            "assets activity bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn assets_activity_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = assets_activity_pane_data(UiSize::new(1280.0, 820.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert!(
        !nodes.is_empty(),
        "assets activity projection should produce template mount nodes"
    );

    let toolbar = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityToolbarPanel")
        .expect("toolbar panel node");
    assert_eq!(toolbar.role.to_string(), "Mount");
    assert!(toolbar.frame.width > 0.0 && toolbar.frame.height > 0.0);

    let tree = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreePanel")
        .expect("tree panel node");
    let tree_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeTitleText")
        .expect("tree title node");
    let tree_subtitle = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeSubtitleText")
        .expect("tree subtitle node");
    let tree_scroll_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeScrollBody")
        .expect("tree scroll body node");
    let title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTitleText")
        .expect("title node");
    let open_browser = nodes
        .iter()
        .find(|node| node.control_id == "OpenAssetBrowser")
        .expect("open browser button node");
    let search = nodes
        .iter()
        .find(|node| node.control_id == "SearchEdited")
        .expect("search field node");
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("content panel node");
    let utility = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityUtilityPanel")
        .expect("utility panel node");
    let utility_selection = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivitySelectionText")
        .expect("utility selection node");
    let utility_divider = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityUtilityDivider")
        .expect("utility divider node");
    let preview_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewPanel")
        .expect("preview panel node");
    let preview_visual = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewVisualPanel")
        .expect("preview visual node");
    let preview_name = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewNameText")
        .expect("preview name node");
    let reference_left_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftTitleText")
        .expect("left references title node");
    let reference_left_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftScrollBody")
        .expect("left references body node");
    let references_right = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightPanel")
        .expect("right references node");
    let reference_right_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightTitleText")
        .expect("right references title node");

    assert_eq!(title.text.to_string(), "Assets");
    assert_eq!(tree_title.text.to_string(), "Folders");
    assert_eq!(tree_subtitle.text.to_string(), "Project/assets");
    assert_eq!(tree_scroll_body.role.to_string(), "Panel");
    assert_eq!(tree_scroll_body.surface_variant.to_string(), "scroll-body");
    assert_eq!(open_browser.role.to_string(), "Button");
    assert_eq!(open_browser.dispatch_kind.to_string(), "asset");
    assert_eq!(search.role.to_string(), "Mount");
    assert!(tree.frame.x >= toolbar.frame.x);
    assert!(content.frame.x >= tree.frame.x + tree.frame.width);
    assert!(utility.frame.y >= tree.frame.y + tree.frame.height);
    assert!(utility_selection.frame.width > 0.0);
    assert!(utility_divider.frame.height > 0.0);
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
    assert!(references_right.frame.width > 0.0 && references_right.frame.height > 0.0);
}
