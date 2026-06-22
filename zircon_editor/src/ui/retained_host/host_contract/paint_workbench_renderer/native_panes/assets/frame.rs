use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::{frame_from_template, translated};

pub(super) fn asset_tree_row_frame(
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
