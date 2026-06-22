mod row;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, PaneData, TemplatePaneNodeData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use row::draw_asset_tree_hover_row_overlay;

const ACTIVITY_ASSET_TREE_ROW_CONTROL: &str = "AssetsActivityTreeRowPanel";
const BROWSER_ASSET_TREE_ROW_CONTROL: &str = "AssetBrowserSourcesRowPanel";

pub(in crate::ui::retained_host::host_contract) fn draw_activity_asset_tree_hover_overlay(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    draw_asset_tree_hover_overlay(
        frame,
        &pane.assets_activity.nodes,
        body,
        clip,
        ACTIVITY_ASSET_TREE_ROW_CONTROL,
        interaction.activity_asset_tree_hovered_index,
        interaction.activity_asset_tree_scroll_px,
    )
}

pub(in crate::ui::retained_host::host_contract) fn draw_browser_asset_tree_hover_overlay(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    draw_asset_tree_hover_overlay(
        frame,
        &pane.asset_browser.nodes,
        body,
        clip,
        BROWSER_ASSET_TREE_ROW_CONTROL,
        interaction.browser_asset_tree_hovered_index,
        interaction.browser_asset_tree_scroll_px,
    )
}

fn draw_asset_tree_hover_overlay(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    clip: &FrameRect,
    row_control_id: &str,
    hovered_index: i32,
    scroll_px: f32,
) -> bool {
    draw_asset_tree_hover_row_overlay(
        frame,
        nodes,
        body,
        clip,
        row_control_id,
        hovered_index,
        scroll_px,
    )
}
