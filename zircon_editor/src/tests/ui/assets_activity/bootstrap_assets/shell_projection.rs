use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn assets_activity_tree_shell_uses_the_standardized_panel_surface_metrics() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot::default(),
        UiSize::new(1280.0, 820.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    let header = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeHeaderPanel");
    let scroll_body = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeScrollBody");
    let row = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreeRowPanel");
    assert!(header.is_some(), "assets tree header panel node");
    assert!(scroll_body.is_some(), "assets tree scroll body node");
    assert!(row.is_some(), "assets tree row panel node");
    let (Some(header), Some(scroll_body), Some(row)) = (header, scroll_body, row) else {
        return;
    };

    assert_eq!(header.surface_variant.to_string(), "inset");
    assert_eq!(header.corner_radius, 4.0);
    assert_eq!(header.border_width, 1.0);
    assert_eq!(scroll_body.surface_variant.to_string(), "inset");
    assert_eq!(scroll_body.corner_radius, 4.0);
    assert_eq!(scroll_body.border_width, 1.0);
    assert_eq!(row.corner_radius, 4.0);
    assert_eq!(row.border_width, 1.0);
    assert!(row.frame.x >= scroll_body.frame.x);
    assert!(row.frame.y >= scroll_body.frame.y);
    assert!(
        row.frame.x + row.frame.width <= scroll_body.frame.x + scroll_body.frame.width,
        "tree row should remain inside the scroll body"
    );
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
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.control_id == "SearchEdited")
            .count(),
        1,
        "TextField projection should not keep both component and generated text nodes"
    );
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
        .find(|node| node.control_id == "AssetsActivityPreviewToolkitText")
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
    let kind_filter = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityKindFilterDropdown")
        .expect("kind filter dropdown node");

    assert_eq!(title.text.to_string(), "Assets");
    assert_eq!(tree_title.text.to_string(), "Folders");
    assert_eq!(tree_subtitle.text.to_string(), "Browse project assets");
    assert_eq!(tree_scroll_body.role.to_string(), "Panel");
    assert_eq!(tree_scroll_body.surface_variant.to_string(), "inset");
    assert_eq!(open_browser.role.to_string(), "IconButton");
    assert_eq!(open_browser.component_role.to_string(), "icon-button");
    assert_eq!(open_browser.text.to_string(), "");
    assert_eq!(open_browser.icon_name.to_string(), "folder-open-outline");
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
    assert_eq!(preview_adapter.text.to_string(), "No toolkit");
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
    assert_eq!(thumb_mode.role.to_string(), "IconButton");
    assert_eq!(thumb_mode.component_role.to_string(), "icon-button");
    assert_eq!(thumb_mode.text.to_string(), "");
    assert_eq!(thumb_mode.icon_name.to_string(), "grid-outline");
    assert_eq!(thumb_mode.surface_variant.to_string(), "inset");
    assert_eq!(thumb_mode.dispatch_kind.to_string(), "asset");
    assert_eq!(
        thumb_mode.action_id.to_string(),
        "workbench.asset.view_mode.set"
    );
    assert_eq!(
        thumb_mode.binding_id.to_string(),
        "AssetSurface/SetViewMode"
    );
    assert_eq!(thumb_mode.value_text.to_string(), "thumbnail");
    assert!(references_tab.selected);
    assert_eq!(references_tab.surface_variant.to_string(), "inset");
    assert_eq!(references_tab.dispatch_kind.to_string(), "asset");
    assert_eq!(
        references_tab.action_id.to_string(),
        "workbench.asset.utility_tab.set"
    );
    assert_eq!(
        references_tab.binding_id.to_string(),
        "AssetSurface/SetUtilityTab"
    );
    assert_eq!(references_tab.value_text.to_string(), "references");
    assert_eq!(kind_filter.role.to_string(), "Dropdown");
    assert_eq!(kind_filter.component_role.to_string(), "dropdown");
    assert_eq!(kind_filter.value_text.to_string(), "Textures");
    assert_eq!(kind_filter.dispatch_kind.to_string(), "asset");
    assert_eq!(kind_filter.options.row_count(), 16);
    assert!(kind_filter
        .options
        .iter()
        .any(|option| option.as_str() == "Texture|label=Textures,selected"));
    assert_eq!(kind_filter.action_id.to_string(), "");
    assert_eq!(
        kind_filter.binding_id.to_string(),
        "AssetSurface/SetKindFilter"
    );
    assert_eq!(
        kind_filter.edit_action_id.to_string(),
        "AssetSurface/SetKindFilter"
    );
    assert!(references_right.frame.width > 0.0 && references_right.frame.height > 0.0);
}
