use super::super::asset_browser_pane_nodes;
use super::find_node;
use crate::ui::workbench::asset_content_layout::{
    AssetContentPaintMetadata, AssetContentRect, BrowserAssetReferenceListKind,
};
use crate::ui::workbench::snapshot::{
    AssetReferenceSnapshot, AssetSelectionSnapshot, AssetUtilityTab, AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn references_tab_builds_all_snapshot_rows_into_responsive_viewports() {
    let nodes = asset_browser_pane_nodes(&reference_snapshot(), UiSize::new(900.0, 620.0));
    let content = find_node(&nodes, "AssetBrowserUtilityContentPanel");
    let left = find_node(&nodes, "AssetBrowserReferenceLeftPanel");
    let right = find_node(&nodes, "AssetBrowserReferenceRightPanel");
    let left_scroll = find_node(&nodes, "AssetBrowserReferenceLeftScrollBody");
    let right_scroll = find_node(&nodes, "AssetBrowserReferenceRightScrollBody");
    let first = find_node(&nodes, "AssetBrowserReferenceLeftRowPanel01");
    let second = find_node(&nodes, "AssetBrowserReferenceLeftRowPanel02");
    let used_by = find_node(&nodes, "AssetBrowserReferenceRightRowPanel01");

    assert_eq!(
        find_node(&nodes, "AssetBrowserReferenceLeftTitleText").text,
        "References (2)"
    );
    assert_eq!(
        find_node(&nodes, "AssetBrowserReferenceRightTitleText").text,
        "Used By (1)"
    );
    assert_eq!(left.frame.x, content.frame.x);
    assert_eq!(left.frame.width, right.frame.width);
    assert!(right.frame.x >= left.frame.x + left.frame.width);
    assert!(right.frame.x + right.frame.width <= content.frame.x + content.frame.width);
    assert_eq!(second.frame.y - first.frame.y, 38.0);
    assert!(first.frame.y >= left_scroll.frame.y);
    assert!(used_by.frame.y >= right_scroll.frame.y);
    assert!(
        !first.selected && !second.selected && !used_by.selected,
        "activating the References tab must not select every reference-row surface"
    );
    assert!(!first.focused && !second.focused && !used_by.focused);

    let metadata = nodes
        .metadata::<AssetContentPaintMetadata>()
        .expect("browser metadata");
    assert_eq!(
        metadata.browser_reference_viewport(BrowserAssetReferenceListKind::References),
        Some(AssetContentRect {
            x: left_scroll.frame.x,
            y: left_scroll.frame.y,
            width: left_scroll.frame.width,
            height: left_scroll.frame.height,
        })
    );
    assert_eq!(
        metadata.browser_reference_row_count(BrowserAssetReferenceListKind::References),
        2
    );
    assert_eq!(
        metadata.browser_reference_row_count(BrowserAssetReferenceListKind::UsedBy),
        1
    );
}

#[test]
fn references_tab_stacks_lists_inside_narrow_real_browser_content() {
    let nodes = asset_browser_pane_nodes(&reference_snapshot(), UiSize::new(300.0, 620.0));
    let content = find_node(&nodes, "AssetBrowserUtilityContentPanel");
    let left = find_node(&nodes, "AssetBrowserReferenceLeftPanel");
    let right = find_node(&nodes, "AssetBrowserReferenceRightPanel");

    assert_eq!(left.frame.x, content.frame.x);
    assert_eq!(right.frame.x, content.frame.x);
    assert_eq!(left.frame.width, content.frame.width);
    assert_eq!(right.frame.width, content.frame.width);
    assert!(right.frame.y >= left.frame.y + left.frame.height);
    assert!(right.frame.y + right.frame.height <= content.frame.y + content.frame.height);
}

fn reference_snapshot() -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        utility_tab: AssetUtilityTab::References,
        selection: AssetSelectionSnapshot {
            references: vec![
                asset_reference("material", "M_Brick", "Content/Materials/M_Brick"),
                asset_reference("texture", "T_Brick", "Content/Textures/T_Brick"),
            ],
            used_by: vec![asset_reference(
                "scene",
                "DemoScene",
                "Content/Scenes/DemoScene",
            )],
            ..AssetSelectionSnapshot::default()
        },
        ..AssetWorkspaceSnapshot::default()
    }
}

fn asset_reference(uuid: &str, display_name: &str, locator: &str) -> AssetReferenceSnapshot {
    AssetReferenceSnapshot {
        uuid: uuid.to_string(),
        display_name: display_name.to_string(),
        locator: locator.to_string(),
        ..AssetReferenceSnapshot::default()
    }
}
