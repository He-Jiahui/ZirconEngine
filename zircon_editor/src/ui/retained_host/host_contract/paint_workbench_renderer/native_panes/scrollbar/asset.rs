use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};

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

fn matches_asset_tree_row(control_id: &str, row_control_id: &str) -> bool {
    control_id
        .rsplit('/')
        .next()
        .is_some_and(|leaf| leaf == row_control_id)
}
