use crate::ui::layouts::views::ViewTemplateNodeData;
use zircon_runtime_interface::ui::layout::UiSize;

use super::toolbar_layout::AssetBrowserToolbarLayout;

const STANDARD_PANEL_GAP: f32 = 6.0;

pub(super) fn apply_asset_browser_standard_stack_layout(
    nodes: &mut [ViewTemplateNodeData],
    size: UiSize,
    toolbar_layout: &AssetBrowserToolbarLayout,
) {
    let Some(main) = node_frame(nodes, "AssetBrowserMainPanel") else {
        return;
    };
    let Some(utility) = node_frame(nodes, "AssetBrowserUtilityPanel") else {
        return;
    };

    let viewport_height = size.height.max(0.0);
    let utility_y = utility
        .y
        .min((viewport_height - utility.height).max(utility.y));
    let main_y = toolbar_layout.main_y;
    let main_height = (utility_y - STANDARD_PANEL_GAP - main_y).max(main.height);
    let delta_y = main_y - main.y;
    let delta_height = main_height - main.height;

    for node in nodes {
        let control_id = node.control_id.as_str();
        if control_id == "AssetBrowserMainPanel" {
            node.frame.y = main_y;
            node.frame.height = main_height;
            continue;
        }
        if !is_asset_browser_main_stack_control(control_id) {
            continue;
        }
        node.frame.y += delta_y;
        if is_stretchable_main_stack_surface(control_id) {
            node.frame.height = (node.frame.height + delta_height).max(0.0);
        }
    }
}

fn is_asset_browser_main_stack_control(control_id: &str) -> bool {
    control_id.starts_with("AssetBrowserSources")
        || control_id.starts_with("AssetBrowserContent")
        || control_id.starts_with("AssetBrowserDetails")
        || control_id.starts_with("WorkbenchAssetBrowser")
}

fn is_stretchable_main_stack_surface(control_id: &str) -> bool {
    matches!(
        control_id,
        "AssetBrowserSourcesPanel"
            | "AssetBrowserSourcesScrollBody"
            | "AssetBrowserContentPanel"
            | "AssetBrowserAssetTablePanel"
            | "AssetBrowserDetailsPanel"
            | "AssetBrowserDetailsScrollBody"
            | "AssetBrowserDetailsContentPanel"
    )
}

fn node_frame(
    nodes: &[ViewTemplateNodeData],
    control_id: &str,
) -> Option<crate::ui::layouts::views::ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
}
