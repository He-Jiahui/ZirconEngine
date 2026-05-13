use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/assets_activity.v2.ui.toml"
));

#[test]
fn assets_activity_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(ASSETS_ACTIVITY_LAYOUT_TOML)
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
            layout.nodes.contains_key(required_node),
            "assets activity bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn assets_activity_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            utility_tab: AssetUtilityTab::References,
            kind_filter: Some(ResourceKind::Texture),
            selected_folder_id: Some("res://textures".to_string()),
            selected_asset_uuid: Some("33333333-3333-3333-3333-333333333333".to_string()),
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(1280.0, 820.0),
    );
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
    let preview_locator = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewLocatorText")
        .expect("preview locator node");
    let preview_kind = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewKindText")
        .expect("preview kind node");
    let preview_identity = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewIdentityText")
        .expect("preview identity node");
    let preview_adapter = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityPreviewAdapterText")
        .expect("preview adapter node");
    let reference_left_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftTitleText")
        .expect("left references title node");
    let reference_left_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftScrollBody")
        .expect("left references body node");
    let reference_left_row = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceLeftRowPanel")
        .expect("left references row node");
    let references_right = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightPanel")
        .expect("right references node");
    let reference_right_title = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightTitleText")
        .expect("right references title node");
    let reference_right_row = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightRowPanel")
        .expect("right references row node");
    let thumb_mode = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityViewModeThumbButton")
        .expect("thumb mode node");
    let references_tab = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferencesTabButton")
        .expect("references tab node");
    let texture_chip = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityKindTextureChip")
        .expect("texture chip node");

    assert_eq!(title.text.to_string(), "Assets");
    assert_eq!(tree_title.text.to_string(), "Folders");
    assert_eq!(tree_subtitle.text.to_string(), "Browse project assets");
    assert_eq!(tree_scroll_body.role.to_string(), "Panel");
    assert_eq!(tree_scroll_body.surface_variant.to_string(), "scroll-body");
    assert_eq!(open_browser.role.to_string(), "Button");
    assert_eq!(open_browser.dispatch_kind.to_string(), "asset");
    assert_eq!(
        open_browser.binding_id.to_string(),
        "AssetSurface/OpenAssetBrowser"
    );
    assert_eq!(search.role.to_string(), "InputField");
    assert_eq!(search.component_role.to_string(), "input-field");
    assert_eq!(search.text.to_string(), "Search");
    assert_eq!(search.value_text.to_string(), "");
    assert_eq!(search.dispatch_kind.to_string(), "asset");
    assert_eq!(search.binding_id.to_string(), "AssetSurface/SearchEdited");
    assert_eq!(
        search.edit_action_id.to_string(),
        "AssetSurface/SearchEdited"
    );
    assert_eq!(search.commit_action_id.to_string(), "");
    assert!(tree.frame.x >= toolbar.frame.x);
    assert!(content.frame.x >= tree.frame.x + tree.frame.width);
    assert!(utility.frame.y >= tree.frame.y + tree.frame.height);
    assert!(utility_selection.frame.width > 0.0);
    assert!(utility_divider.frame.height > 0.0);
    assert_eq!(preview_panel.role.to_string(), "Panel");
    assert_eq!(preview_panel.surface_variant.to_string(), "asset-preview");
    assert!(!preview_panel.selected);
    assert_eq!(preview_visual.role.to_string(), "Panel");
    assert!(!preview_visual.selected);
    assert_eq!(
        preview_visual.surface_variant.to_string(),
        "asset-preview-visual"
    );
    assert_eq!(preview_name.text.to_string(), "No Asset Selected");
    assert_eq!(preview_locator.text.to_string(), "No project locator");
    assert_eq!(preview_kind.text.to_string(), "Unknown Type");
    assert_eq!(preview_identity.text.to_string(), "No UUID");
    assert_eq!(preview_adapter.text.to_string(), "No adapter");
    assert_eq!(reference_left_title.text.to_string(), "References");
    assert_eq!(reference_left_body.role.to_string(), "Panel");
    assert_eq!(
        reference_left_body.surface_variant.to_string(),
        "scroll-body"
    );
    assert!(reference_left_row.selected);
    assert_eq!(reference_right_title.text.to_string(), "Used By");
    assert!(reference_right_row.selected);
    assert!(!preview_name.selected);
    assert_eq!(preview_name.text_tone.to_string(), "muted");
    assert!(!preview_locator.selected);
    assert_eq!(preview_locator.text_tone.to_string(), "muted");
    assert!(!preview_kind.selected);
    assert_eq!(preview_kind.text_tone.to_string(), "muted");
    assert!(!preview_identity.selected);
    assert_eq!(preview_identity.text_tone.to_string(), "muted");
    assert!(!preview_adapter.selected);
    assert_eq!(preview_adapter.text_tone.to_string(), "muted");
    assert!(thumb_mode.selected);
    assert_eq!(thumb_mode.surface_variant.to_string(), "inset");
    assert_eq!(thumb_mode.dispatch_kind.to_string(), "asset");
    assert_eq!(thumb_mode.action_id.to_string(), "SetViewMode");
    assert_eq!(
        thumb_mode.binding_id.to_string(),
        "AssetSurface/SetViewMode"
    );
    assert_eq!(thumb_mode.value_text.to_string(), "thumbnail");
    assert!(references_tab.selected);
    assert_eq!(references_tab.surface_variant.to_string(), "inset");
    assert_eq!(references_tab.dispatch_kind.to_string(), "asset");
    assert_eq!(references_tab.action_id.to_string(), "SetUtilityTab");
    assert_eq!(
        references_tab.binding_id.to_string(),
        "AssetSurface/SetUtilityTab"
    );
    assert_eq!(references_tab.value_text.to_string(), "references");
    assert!(texture_chip.selected);
    assert_eq!(texture_chip.surface_variant.to_string(), "inset");
    assert_eq!(texture_chip.dispatch_kind.to_string(), "asset");
    assert_eq!(texture_chip.action_id.to_string(), "SetKindFilter");
    assert_eq!(
        texture_chip.binding_id.to_string(),
        "AssetSurface/SetKindFilter"
    );
    assert_eq!(texture_chip.value_text.to_string(), "Texture");
    assert!(references_right.frame.width > 0.0 && references_right.frame.height > 0.0);
}
