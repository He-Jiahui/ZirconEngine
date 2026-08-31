use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile,
};
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/assets_activity.zui"
));

#[test]
fn assets_activity_static_text_uses_central_typography_tokens() {
    for token in [
        "$editor.typography.body.size",
        "$editor.typography.caption.size",
        "$editor.typography.overlay.size",
        "$editor.typography.strong.weight",
        "$editor.typography.emphasis.weight",
    ] {
        assert!(
            ASSETS_ACTIVITY_LAYOUT_TOML.contains(token),
            "assets activity text should reference the central token `{token}`"
        );
    }
    for raw_value in [
        "font_size = 9.0",
        "font_size = 10.0",
        "font_size = 11.0",
        "font_size = 12.0",
        "font_weight = 600",
        "font_weight = 700",
    ] {
        assert!(
            !ASSETS_ACTIVITY_LAYOUT_TOML.contains(raw_value),
            "assets activity must not retain a local typography value `{raw_value}`"
        );
    }
}

#[test]
fn assets_activity_standard_container_metrics_use_central_tokens() {
    for token in [
        "$editor.control.border_width",
        "$editor.control.height.default",
        "$editor.control.height.dense",
        "$editor.control.radius.small",
        "$editor.density.gap.xsmall",
        "$editor.density.gap.small",
        "$editor.density.gap.medium",
        "$editor.density.gap.large",
        "$editor.density.row_height",
    ] {
        assert!(
            ASSETS_ACTIVITY_LAYOUT_TOML.contains(token),
            "assets activity containers should reference the central token `{token}`"
        );
    }
    for raw_value in [
        "radius = 3.0",
        "radius = 4.0",
        "border_width = 1.0",
        "gap = 2.0",
        "gap = 3.0",
        "gap = 4.0",
        "gap = 8.0",
        "gap = 10.0",
        "gap = 12.0",
        "height = { min = 28.0, preferred = 28.0, max = 28.0",
        "height = { min = 32.0, preferred = 32.0, max = 32.0",
    ] {
        assert!(
            !ASSETS_ACTIVITY_LAYOUT_TOML.contains(raw_value),
            "assets activity must not retain a local container metric `{raw_value}`"
        );
    }
}

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
        "toolbar_filter_row",
        "toolbar_kind_filter_dropdown",
        "toolbar_view_mode_list_button",
        "toolbar_view_mode_thumb_button",
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

#[test]
fn assets_activity_regular_drawer_compacts_toolbar_and_reclaims_content_width() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::Thumbnail,
            utility_tab: AssetUtilityTab::Preview,
            kind_filter: Some(ResourceKind::Texture),
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing compact Assets activity node `{control_id}`"))
    };

    let toolbar = node("AssetsActivityToolbarPanel");
    let browser = node("OpenAssetBrowser");
    let search = node("SearchEdited");
    let kind_filter = node("AssetsActivityKindFilterDropdown");
    let list = node("AssetsActivityViewModeListButton");
    let thumb = node("AssetsActivityViewModeThumbButton");
    let tree = node("AssetsActivityTreePanel");
    let content = node("AssetsActivityContentPanel");
    let main = node("AssetsActivityMainPanel");
    let selection = node("AssetsActivitySelectionText");
    let preview = node("AssetsActivityPreviewTabButton");
    let references = node("AssetsActivityReferencesTabButton");

    assert_eq!(browser.frame.width, 28.0);
    assert_eq!(list.frame.width, 28.0);
    assert_eq!(thumb.frame.width, 28.0);
    assert_eq!(thumb.text.to_string(), "");
    assert_eq!(kind_filter.value_text.to_string(), "Textures");
    assert_eq!(kind_filter.options.row_count(), 16);
    assert!(kind_filter
        .options
        .iter()
        .any(|option| option.as_str() == "Texture|label=Textures,selected"));
    assert_eq!(preview.text.to_string(), "Preview");
    assert!(preview.selected);
    assert!(toolbar.frame.height <= 68.0);
    assert!(search.frame.x + search.frame.width <= browser.frame.x);
    assert!(kind_filter.frame.x + kind_filter.frame.width <= list.frame.x);
    assert!(list.frame.x + list.frame.width <= thumb.frame.x);
    for control in [
        browser,
        search,
        kind_filter,
        list,
        thumb,
        preview,
        references,
    ] {
        assert!(
            control.frame.width > 0.0,
            "compact control should remain visible: {control:?}"
        );
        assert!(
            control.frame.x + control.frame.width <= 226.0 + f32::EPSILON,
            "compact control should stay inside the drawer: {control:?}"
        );
    }
    assert_eq!(tree.frame.width, 0.0);
    assert!(content.frame.width >= 210.0);
    assert!(main.frame.height >= 120.0);
    assert_eq!(selection.frame.width, 0.0);
}

#[test]
fn assets_activity_regular_drawer_references_use_one_readable_summary_column() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            utility_tab: AssetUtilityTab::References,
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing compact reference node `{control_id}`"))
    };
    let left = frame("AssetsActivityReferenceLeftPanel");
    let right = frame("AssetsActivityReferenceRightPanel");
    let summary = frame("AssetsActivityReferenceLeftEmptyText");

    assert!(left.width >= 210.0);
    assert_eq!(right.width, 0.0);
    assert!(summary.width > 0.0);
    assert!(summary.x + summary.width <= 226.0 + f32::EPSILON);
}

#[test]
fn wide_assets_activity_utility_uses_mutually_exclusive_relative_composites() {
    let preview_pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            utility_tab: AssetUtilityTab::Preview,
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(1280.0, 820.0),
    );
    let preview_nodes = (0..preview_pane.nodes.row_count())
        .filter_map(|row| preview_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let preview_node = |control_id: &str| {
        preview_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing wide preview node `{control_id}`"))
    };
    let preview = preview_node("AssetsActivityPreviewPanel");
    let preview_visual = preview_node("AssetsActivityPreviewVisualPanel");
    let references_left = preview_node("AssetsActivityReferenceLeftPanel");
    let references_right = preview_node("AssetsActivityReferenceRightPanel");

    assert!(preview.frame.width > 0.0 && preview.frame.height > 0.0);
    assert_eq!(references_left.frame.width, 0.0);
    assert_eq!(references_right.frame.width, 0.0);
    assert!(preview_visual.frame.x >= preview.frame.x);
    assert!(preview_visual.frame.y >= preview.frame.y);
    assert!(
        preview_visual.frame.x + preview_visual.frame.width
            <= preview.frame.x + preview.frame.width
    );
    assert!(
        preview_visual.frame.y + preview_visual.frame.height
            <= preview.frame.y + preview.frame.height
    );

    let preview_text = [
        "AssetsActivityPreviewNameText",
        "AssetsActivityPreviewLocatorText",
        "AssetsActivityPreviewKindText",
        "AssetsActivityPreviewIdentityText",
        "AssetsActivityPreviewToolkitText",
        "AssetsActivityPreviewMetaPathText",
        "AssetsActivityPreviewDiagnosticsText",
    ]
    .map(preview_node);
    for text in preview_text {
        assert!(text.frame.x >= preview.frame.x);
        assert!(text.frame.y >= preview.frame.y);
        assert!(text.frame.x + text.frame.width <= preview.frame.x + preview.frame.width);
        assert!(text.frame.y + text.frame.height <= preview.frame.y + preview.frame.height);
    }
    for pair in preview_text.windows(2) {
        assert!(
            pair[0].frame.y + pair[0].frame.height <= pair[1].frame.y,
            "preview text slots must not overlap: {:?} then {:?}",
            pair[0].control_id,
            pair[1].control_id
        );
    }

    let references_pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot {
            utility_tab: AssetUtilityTab::References,
            ..AssetWorkspaceSnapshot::default()
        },
        UiSize::new(1280.0, 820.0),
    );
    let reference_nodes = (0..references_pane.nodes.row_count())
        .filter_map(|row| references_pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let reference_node = |control_id: &str| {
        reference_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing wide references node `{control_id}`"))
    };
    let hidden_preview = reference_node("AssetsActivityPreviewPanel");
    let left = reference_node("AssetsActivityReferenceLeftPanel");
    let right = reference_node("AssetsActivityReferenceRightPanel");
    let left_title = reference_node("AssetsActivityReferenceLeftTitleText");
    let left_body = reference_node("AssetsActivityReferenceLeftScrollBody");
    let right_title = reference_node("AssetsActivityReferenceRightTitleText");
    let right_body = reference_node("AssetsActivityReferenceRightScrollBody");

    assert_eq!(hidden_preview.frame.width, 0.0);
    assert!(left.frame.width > 0.0 && right.frame.width > 0.0);
    assert!(left.frame.x + left.frame.width <= right.frame.x);
    assert!(left_title.frame.y + left_title.frame.height <= left_body.frame.y);
    assert!(right_title.frame.y + right_title.frame.height <= right_body.frame.y);
    assert!(left_body.frame.y + left_body.frame.height <= left.frame.y + left.frame.height);
    assert!(right_body.frame.y + right_body.frame.height <= right.frame.y + right.frame.height);
}

#[test]
fn assets_activity_content_rows_share_the_activity_pointer_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        selected_asset_uuid: Some("asset-selected".to_string()),
        visible_assets: vec![
            activity_asset(
                "asset-first",
                "editor_base.zui",
                ResourceKind::UiStyle,
                false,
            ),
            activity_asset(
                "asset-selected",
                "workbench_page_chrome.zui",
                ResourceKind::UiLayout,
                true,
            ),
        ]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 346.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing Assets activity content node `{control_id}`"))
    };
    let content = node("AssetsActivityContentPanel");
    let first = node("AssetsActivityContentItemRow00");
    let selected = node("AssetsActivityContentItemRow01");
    let selected_name = node("AssetsActivityContentItemName01");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );

    assert_eq!(first.frame.x, content.frame.x + metrics.row_x);
    assert_eq!(first.frame.y, content.frame.y + metrics.first_row_y());
    assert_eq!(first.frame.width, metrics.row_width(content.frame.width));
    assert_eq!(first.frame.height, metrics.item_height);
    assert_eq!(
        selected.frame.y,
        first.frame.y + metrics.item_height + metrics.row_gap
    );
    assert!(selected.selected);
    assert!(!selected_name.selected);
    assert_eq!(selected_name.border_width, 0.0);
    assert!(selected_name.text.to_string().ends_with(".zui"));
    assert!(selected_name.text.to_string().contains("..."));
    assert!(selected_name.frame.x >= selected.frame.x);
    assert!(
        selected_name.frame.x + selected_name.frame.width
            <= selected.frame.x + selected.frame.width
    );
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "AssetsActivityContentEmptyText"));
}

#[test]
fn empty_assets_activity_content_has_an_explicit_readable_state() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot::default(),
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("content panel");
    let empty = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentEmptyText")
        .expect("explicit empty content state");

    assert_eq!(empty.text.to_string(), "No assets in this folder");
    assert_eq!(empty.text_tone.to_string(), "muted");
    assert!(empty.frame.width > 0.0);
    assert!(empty.frame.x + empty.frame.width <= content.frame.x + content.frame.width);
}

#[test]
fn short_assets_activity_drawer_preserves_one_complete_asset_row_before_preview() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: vec![activity_asset(
            "asset-visible",
            "workbench_page_chrome.zui",
            ResourceKind::UiLayout,
            true,
        )]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing short drawer node `{control_id}`"))
    };
    let content = frame("AssetsActivityContentPanel");
    let row = frame("AssetsActivityContentItemRow00");
    let utility = frame("AssetsActivityUtilityPanel");
    let preview = frame("AssetsActivityPreviewPanel");

    assert!(row.width > 0.0 && row.height > 0.0);
    assert!(row.y + row.height <= content.y + content.height + f32::EPSILON);
    assert!(content.y + content.height <= utility.y + f32::EPSILON);
    assert!(preview.width > 0.0 && preview.height > 0.0);
}

#[test]
fn short_assets_activity_keeps_below_viewport_rows_in_scroll_source_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: (0..5)
            .map(|index| {
                activity_asset(
                    &format!("asset-{index}"),
                    &format!("workbench_asset_{index}.zui"),
                    ResourceKind::UiLayout,
                    false,
                )
            })
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing short scroll-source node `{control_id}`"))
    };
    let content_node = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("short content panel");
    let content = content_node.frame.clone();
    let late_row = frame("AssetsActivityContentItemRow04");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );

    assert!(late_row.width > 0.0 && late_row.height > 0.0);
    assert!(late_row.y >= content.y + content.height);
    assert_eq!(content_node.value_number, metrics.list_height(0, 5));
}

fn activity_asset(
    uuid: &str,
    display_name: &str,
    kind: ResourceKind,
    selected: bool,
) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: format!("res://ui/{display_name}"),
        display_name: display_name.to_string(),
        file_name: display_name.to_string(),
        extension: display_name
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or_default()
            .to_string(),
        kind,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            kind,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(1),
    }
}
