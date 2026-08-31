use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_geometry::intersect;
use super::super::super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::super::ACCENT;
use super::super::frame::asset_tree_row_frame;

const ASSET_TREE_ROW_HOVERED: [u8; 4] = [
    PALETTE.surface_selected[0],
    PALETTE.surface_selected[1],
    PALETTE.surface_selected[2],
    120,
];

pub(super) fn draw_asset_tree_hover_row_overlay(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    clip: &FrameRect,
    hovered_index: i32,
    scroll_px: f32,
) -> bool {
    if hovered_index < 0 {
        return false;
    }
    let Some(row) = asset_tree_row_frame(nodes, body, hovered_index as usize, scroll_px.max(0.0))
    else {
        return false;
    };
    if intersect(&row, clip).is_none() {
        return false;
    }
    draw_rect_clipped(frame, row.clone(), Some(clip), ASSET_TREE_ROW_HOVERED);
    draw_border_clipped(frame, row, Some(clip), ACCENT);
    true
}
