use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

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
