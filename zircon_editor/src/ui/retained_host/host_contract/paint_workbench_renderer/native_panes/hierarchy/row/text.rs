use crate::ui::retained_host::host_contract::data::{FrameRect, SceneNodeData};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::draw_text_bars_clipped;

use super::super::super::super::MUTED_TEXT;

const HIERARCHY_ROW_INDENT: f32 = 14.0;
const HIERARCHY_ROW_TEXT_X: f32 = 8.0;
const HIERARCHY_ROW_TEXT_Y: f32 = 4.0;

pub(super) fn draw_hierarchy_row_text(
    frame: &mut HostRgbaFrame,
    row: &FrameRect,
    node: &SceneNodeData,
    row_clip: &FrameRect,
) {
    draw_text_bars_clipped(
        frame,
        row_text_x(row, node),
        row.y + HIERARCHY_ROW_TEXT_Y,
        &node.name,
        Some(row_clip),
        MUTED_TEXT,
    );
}

fn row_text_x(row: &FrameRect, node: &SceneNodeData) -> f32 {
    let indent = node.depth.max(0) as f32 * HIERARCHY_ROW_INDENT;
    row.x + HIERARCHY_ROW_TEXT_X + indent.min(row.width * 0.5)
}
