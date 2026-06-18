use crate::ui::retained_host::hierarchy_pointer::constants::{
    ROW_GAP, ROW_HEIGHT, ROW_WIDTH_INSET, ROW_X, ROW_Y,
};

use super::super::super::data::{FrameRect, HostPaneInteractionStateData, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{
    frame_from_template, intersect, is_visible_frame, translated,
};
use super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::paint_theme::PALETTE;
use super::super::{ACCENT, MUTED_TEXT};

const HIERARCHY_ROW: [u8; 4] = PALETTE.surface;
const HIERARCHY_ROW_HOVERED: [u8; 4] = PALETTE.surface_hover;
const HIERARCHY_ROW_SELECTED: [u8; 4] = PALETTE.surface_selected;
const HIERARCHY_ROW_INDENT: f32 = 14.0;
const HIERARCHY_ROW_TEXT_X: f32 = 8.0;
const HIERARCHY_ROW_TEXT_Y: f32 = 4.0;

pub(super) fn draw_hierarchy_rows(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let node_count = pane.hierarchy.hierarchy_nodes.row_count();
    if node_count == 0 {
        return false;
    }
    let viewport = hierarchy_viewport_frame(pane, body);
    let Some(row_clip) = intersect(&viewport, clip) else {
        return false;
    };
    let row_width = (viewport.width - ROW_WIDTH_INSET).max(0.0);
    let scroll_px = interaction.hierarchy_scroll_px.max(0.0);

    for index in 0..node_count {
        let Some(node) = pane.hierarchy.hierarchy_nodes.row_data(index) else {
            continue;
        };
        let row = FrameRect {
            x: viewport.x + ROW_X,
            y: viewport.y + ROW_Y + index as f32 * (ROW_HEIGHT + ROW_GAP) - scroll_px,
            width: row_width,
            height: ROW_HEIGHT,
        };
        if intersect(&row, &row_clip).is_none() {
            continue;
        }
        let color = if interaction.hovered_hierarchy_index == index as i32 {
            HIERARCHY_ROW_HOVERED
        } else if node.selected {
            HIERARCHY_ROW_SELECTED
        } else {
            HIERARCHY_ROW
        };
        draw_rect_clipped(frame, row.clone(), Some(&row_clip), color);
        if node.selected {
            draw_border_clipped(frame, row.clone(), Some(&row_clip), ACCENT);
        }
        let indent = node.depth.max(0) as f32 * HIERARCHY_ROW_INDENT;
        draw_text_bars_clipped(
            frame,
            row.x + HIERARCHY_ROW_TEXT_X + indent.min(row.width * 0.5),
            row.y + HIERARCHY_ROW_TEXT_Y,
            &node.name,
            Some(&row_clip),
            MUTED_TEXT,
        );
    }
    true
}

fn hierarchy_viewport_frame(pane: &PaneData, body: &FrameRect) -> FrameRect {
    (0..pane.hierarchy.nodes.row_count())
        .filter_map(|row| pane.hierarchy.nodes.row_data(row))
        .find_map(|node| {
            matches!(
                node.control_id.as_str(),
                "HierarchyListPanel" | "HierarchyTreeSlotAnchor"
            )
            .then(|| translated(&frame_from_template(&node.frame), body.x, body.y))
            .filter(is_visible_frame)
        })
        .unwrap_or_else(|| body.clone())
}
