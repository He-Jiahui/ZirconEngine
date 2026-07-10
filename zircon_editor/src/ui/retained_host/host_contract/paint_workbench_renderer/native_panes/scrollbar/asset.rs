use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::ACTIVITY_CONTENT_PANEL_CONTROL_ID;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::{frame_from_template, translated};

pub(super) fn asset_tree_viewport_frame(body: &FrameRect) -> FrameRect {
    let viewport_y = crate::ui::retained_host::asset_pointer::asset_tree_viewport_y();
    FrameRect {
        x: body.x,
        y: body.y + viewport_y,
        width: body.width,
        height: (body.height - viewport_y).max(0.0),
    }
}

pub(super) fn asset_tree_row_count(
    nodes: &ModelRc<TemplatePaneNodeData>,
    row_control_id: &str,
) -> usize {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .filter(|node| matches_asset_tree_row(node.control_id.as_str(), row_control_id))
        .count()
}

pub(super) fn activity_asset_content_viewport_and_extent(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
) -> Option<(FrameRect, f32)> {
    let panel = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| {
            node.control_id.rsplit('/').next() == Some(ACTIVITY_CONTENT_PANEL_CONTROL_ID)
        })?;
    let viewport = translated(&frame_from_template(&panel.frame), body.x, body.y);
    let extent = if panel.value_number.is_finite() {
        panel.value_number.max(0.0)
    } else {
        0.0
    };
    Some((viewport, extent))
}

fn matches_asset_tree_row(control_id: &str, row_control_id: &str) -> bool {
    control_id
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == row_control_id)
}
