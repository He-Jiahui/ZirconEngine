use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, PaneData, TemplatePaneNodeData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{frame_from_template, intersect, translated};
use super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::super::super::paint_theme::PALETTE;
use super::super::ACCENT;

const ASSET_TREE_ROW_HOVERED: [u8; 4] = [
    PALETTE.surface_selected[0],
    PALETTE.surface_selected[1],
    PALETTE.surface_selected[2],
    120,
];
const ACTIVITY_ASSET_TREE_ROW_CONTROL: &str = "AssetsActivityTreeRowPanel";
const BROWSER_ASSET_TREE_ROW_CONTROL: &str = "AssetBrowserSourcesRowPanel";

pub(super) fn draw_activity_asset_tree_hover_overlay(
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

pub(super) fn draw_browser_asset_tree_hover_overlay(
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
    if hovered_index < 0 {
        return false;
    }
    let Some(row) = asset_tree_row_frame(
        nodes,
        body,
        row_control_id,
        hovered_index as usize,
        scroll_px.max(0.0),
    ) else {
        return false;
    };
    if intersect(&row, clip).is_none() {
        return false;
    }
    draw_rect_clipped(frame, row.clone(), Some(clip), ASSET_TREE_ROW_HOVERED);
    draw_border_clipped(frame, row, Some(clip), ACCENT);
    true
}

fn asset_tree_row_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    row_control_id: &str,
    hovered_index: usize,
    scroll_px: f32,
) -> Option<FrameRect> {
    let mut asset_row_index = 0;
    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !matches_asset_tree_row(node.control_id.as_str(), row_control_id) {
            continue;
        }
        if asset_row_index == hovered_index {
            let mut frame = translated(&frame_from_template(&node.frame), body.x, body.y);
            frame.y -= scroll_px;
            return Some(frame);
        }
        asset_row_index += 1;
    }
    None
}

fn matches_asset_tree_row(control_id: &str, row_control_id: &str) -> bool {
    control_id
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == row_control_id)
}
